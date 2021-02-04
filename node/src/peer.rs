use anyhow::anyhow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::cookie::Cookie;
use crate::header::{Flags, Header, MessageType};
use crate::messages::node_id_handshake::{NodeIdHandshakeQuery, NodeIdHandshakeResponse};
use crate::state::State;
use crate::wire::Wire;

pub struct Peer {
    state: State,
    stream: TcpStream,

    /// A reusable header to reduce allocations.
    header: Header,

    /// Storage that can be shared within this task without reallocating.
    buffer: Vec<u8>,

    tmp: Option<Cookie>,
}

impl Peer {
    pub fn new(state: State, stream: TcpStream) -> Self {
        let network = state.network();
        Self {
            state,
            stream,
            header: Header::new(network, MessageType::NodeIdHandshake, Flags::new()),
            buffer: Vec::with_capacity(1024),
            tmp: None,
        }
    }

    async fn recv<T: Wire>(&mut self) -> anyhow::Result<T> {
        let len = T::len();

        if len > self.buffer.len() {
            self.buffer.resize(len, 0)
        }

        let buffer = &mut self.buffer[0..len];
        let bytes_read = self.stream.read_exact(buffer).await?;
        dbg!(&buffer);
        if bytes_read < len {
            return Err(anyhow!(
                "Received an incorrect amount of bytes. Got: {} Expected: {}",
                bytes_read,
                len,
            ));
        }

        let buffer = &self.buffer[0..len];
        Ok(T::deserialize(&self.state, buffer)?)
    }

    async fn send<T: Wire>(&mut self, message: &T) -> anyhow::Result<()> {
        self.stream.write_all(&message.serialize()).await?;
        Ok(())
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.query_handshake().await?;

        loop {
            let header = self.recv::<Header>().await?;

            match header.message_type() {
                MessageType::Keepalive => todo!(),
                MessageType::Publish => todo!(),
                MessageType::ConfirmReq => todo!(),
                MessageType::ConfirmAck => todo!(),
                MessageType::BulkPull => todo!(),
                MessageType::BulkPush => todo!(),
                MessageType::FrontierReq => todo!(),
                MessageType::NodeIdHandshake => self.handle_node_id_handshake(header).await?,
                MessageType::BulkPullAccount => todo!(),
                MessageType::TelemetryReq => todo!(),
                MessageType::TelemetryAck => todo!(),
            }
        }
    }

    async fn handle_node_id_handshake(&mut self, header: Header) -> anyhow::Result<()> {
        if header.flags().is_query() {
            dbg!("recv handshake query");
            let query = self.recv::<NodeIdHandshakeQuery>().await?;
            dbg!(query);
        }
        if header.flags().is_response() {
            dbg!("recv handshake response");
            let response = self.recv::<NodeIdHandshakeResponse>().await?;
            dbg!(&response);
            let public = response.public;
            let signature = response.signature;

            let cookie = &self.tmp.as_ref().unwrap();

            if !public.verify(&cookie.as_bytes(), &signature) {
                return Err(anyhow!("Invalid signature in node_id_handshake response"));
            }
            dbg!("omgigod it worked");
        }

        todo!()
    }

    async fn query_handshake(&mut self) -> anyhow::Result<()> {
        let mut header = self.header;
        header.reset(MessageType::NodeIdHandshake, *Flags::new().set_query(true));
        self.send(&header).await?;

        let cookie = Cookie::random();
        self.tmp = Some(cookie.clone());
        let handshake_query = NodeIdHandshakeQuery::new(cookie);
        dbg!("sending cookie");
        self.send(&handshake_query).await?;

        Ok(())
    }
}
