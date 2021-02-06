use crate::cookie::Cookie;
use crate::header::{BlockType, Extensions, Header, MessageType};
use crate::messages::handle_confirm_req::HandleConfirmReq;
use crate::messages::node_id_handshake::{NodeIdHandshakeQuery, NodeIdHandshakeResponse};
use crate::peer::Peer;
use crate::state::State;
use crate::wire::Wire;
use anyhow::anyhow;
use feeless::{expect_len, Seed};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// A connection to a single peer.
pub struct Channel {
    pub(crate) state: State,

    // TODO: Both of these into a Communication trait, for ease of testing. e.g.:
    //  * async fn Comm::send() -> Result<()>
    //  * async fn Comm::recv() -> Result<()>
    //  * fn Comm::address() -> String
    //
    // This would also remove Self::buffer.
    // Not sure about the performance problems of having to use async-trait.
    stream: TcpStream,
    pub(crate) peer_addr: SocketAddr,

    /// A reusable header to reduce allocations.
    pub(crate) header: Header,

    /// Storage that can be shared within this task without reallocating.
    /// This is currently only used for the recv buffers.
    buffer: Vec<u8>,
}

impl Channel {
    pub fn new(state: State, stream: TcpStream) -> Self {
        let network = state.network();
        // TODO: Remove unwrap
        let peer_addr = stream.peer_addr().unwrap();
        Self {
            state,
            stream,
            peer_addr,
            header: Header::new(network, MessageType::NodeIdHandshake, Extensions::new()),
            buffer: Vec::with_capacity(1024),
        }
    }

    async fn recv<T: Wire>(&mut self, header: Option<&Header>) -> anyhow::Result<T> {
        let expected_len = T::len();
        if expected_len > self.buffer.len() {
            self.buffer.resize(expected_len, 0)
        }

        let buffer = &mut self.buffer[0..expected_len];
        let bytes_read = self.stream.read_exact(buffer).await?;
        expect_len(bytes_read, expected_len, "Recv packet")?;

        let buffer = &self.buffer[0..expected_len];
        Ok(T::deserialize(header, buffer)?)
    }

    async fn send<T: Wire>(&mut self, message: &T) -> anyhow::Result<()> {
        self.stream.write_all(&message.serialize()).await?;
        Ok(())
    }

    async fn send_header(
        &mut self,
        message_type: MessageType,
        ext: Extensions,
    ) -> anyhow::Result<()> {
        let mut header = self.header;
        header.reset(message_type, ext);
        Ok(self.send(&header).await?)
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.initial_handshake().await?;

        loop {
            let header = self.recv::<Header>(None).await?;
            header.validate(&self.state)?;
            dbg!(&header);

            match header.message_type() {
                MessageType::Keepalive => self.handle_keepalive(header).await?,
                // MessageType::Publish => todo!(),
                MessageType::ConfirmReq => self.handle_confirm_req(header).await?,
                // MessageType::ConfirmAck => todo!(),
                // MessageType::BulkPull => todo!(),
                // MessageType::BulkPush => todo!(),
                // MessageType::FrontierReq => todo!(),
                MessageType::NodeIdHandshake => self.handle_node_id_handshake(header).await?,
                // MessageType::BulkPullAccount => todo!(),
                // MessageType::TelemetryReq => todo!(),
                // MessageType::TelemetryAck => todo!(),
                _ => todo!("{:?}", header),
            }
        }
    }

    async fn handle_keepalive(&mut self, header: Header) -> anyhow::Result<()> {
        for _ in 0..8 {
            let peer = self.recv::<Peer>(Some(&header)).await?;
            dbg!(peer);
        }
        Ok(())
    }

    async fn handle_node_id_handshake(&mut self, header: Header) -> anyhow::Result<()> {
        if header.ext().is_query() {
            let query = self.recv::<NodeIdHandshakeQuery>(Some(&header)).await?;
            // XXX: Hacky code here just to see if it works!
            // TODO: Move into state
            let seed = Seed::random();
            let private = seed.derive(0);
            let public = private.to_public();
            let signature = private.sign(query.cookie().as_bytes())?;

            debug_assert!(public.verify(query.cookie().as_bytes(), &signature));

            let mut header = self.header;
            header.reset(MessageType::NodeIdHandshake, *Extensions::new().response());
            self.send(&header).await?;

            let response = NodeIdHandshakeResponse::new(public, signature);
            dbg!("sending handshake response");
            self.send(&response).await?;
            dbg!("sending handshake response done");
        }

        if header.ext().is_response() {
            let response = self.recv::<NodeIdHandshakeResponse>(Some(&header)).await?;
            let public = response.public;
            let signature = response.signature;

            let cookie = &self.state.cookie_for_socket_addr(&self.peer_addr).await?;

            if !public.verify(&cookie.as_bytes(), &signature) {
                return Err(anyhow!("Invalid signature in node_id_handshake response"));
            }
            dbg!("signature verified");
        }

        Ok(())
    }

    async fn initial_handshake(&mut self) -> anyhow::Result<()> {
        self.send_header(MessageType::NodeIdHandshake, *Extensions::new().query())
            .await?;

        let cookie = Cookie::random();
        self.state
            .set_cookie(self.peer_addr, cookie.clone())
            .await?;
        let handshake_query = NodeIdHandshakeQuery::new(cookie);
        dbg!("sending cookie");
        self.send(&handshake_query).await?;

        Ok(())
    }

    async fn handle_confirm_req(&mut self, header: Header) -> anyhow::Result<()> {
        let hash_pair_count = header.ext().item_count();
        dbg!(&hash_pair_count);

        let data = self.recv::<HandleConfirmReq>(Some(&header)).await?;

        println!("(TODO) confirm_req");
        dbg!(data);

        Ok(())
    }
}
