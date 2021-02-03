use anyhow::anyhow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::cookie::Cookie;
use crate::header::{Flags, Header, MessageType, Network};
use crate::messages::node_id_handshake::{NodeIdHandshakeQuery, NodeIdHandshakeResponse};
use crate::state::State;
use crate::wire::Wire;

pub struct Connection {
    state: State,
    stream: TcpStream,

    /// A reusable header to reduce allocations.
    header: Header,

    /// Storage that can be shared within this task without reallocating.
    buffer: Vec<u8>,
}

impl Connection {
    pub fn new(state: State, stream: TcpStream) -> Self {
        let network = state.network();
        Self {
            state,
            stream,
            header: Header::new(network, MessageType::NodeIdHandshake, Flags::new()),
            buffer: Vec::with_capacity(1024),
        }
    }

    async fn recv<T: Wire>(&mut self) -> anyhow::Result<T> {
        let len = T::len();

        if len > self.buffer.len() {
            self.buffer.resize(len, 0)
        }

        let buffer = &mut self.buffer[0..len];
        let bytes_read = self.stream.read_exact(buffer).await?;
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
            // Expecting a header
            // let header = Header::deserialize(&self.state, self.recv(Header::LENGTH).await?)?;
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

        // let mut buffer = [0; 8];
        // let result = self.stream.read_exact(&mut buffer).await.unwrap();
        // dbg!(result);
        // dbg!(buffer);
        //
        // let mut buffer = [0; 32];
        // let size = self.stream.read_exact(&mut buffer).await.unwrap();
        // // dbg!(query_hash);
        // dbg!(buffer);

        // Assuming this is a NodeIdHandshake

        // let mut buffer = [0; Header::LENGTH];
        // let result = r.read_exact(&mut buffer).await.unwrap();
        // dbg!(&result);
        // let header = Header::deserialize(Network::Live, &buffer).unwrap();
        // dbg!(&header);
        todo!()
    }

    async fn handle_node_id_handshake(&mut self, header: Header) -> anyhow::Result<()> {
        if header.flags().is_query() {
            let query = self.recv::<NodeIdHandshakeQuery>().await?;
            dbg!(query);
        }
        if header.flags().is_response() {
            let response = self.recv::<NodeIdHandshakeResponse>().await?;
            dbg!(response);
        }

        todo!()
    }

    async fn query_handshake(&mut self) -> anyhow::Result<()> {
        let mut header = self.header;
        header.reset(MessageType::NodeIdHandshake, *Flags::new().set_query(true));
        self.send(&header).await?;

        let cookie = Cookie::new();
        let handshake_query = NodeIdHandshakeQuery::new(cookie);
        self.send(&handshake_query).await?;

        Ok(())
    }
}
