#![forbid(unsafe_code)]

mod header;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::header::{Flags, Header, MessageType, Network};
use feeless;
use std::convert::TryFrom;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let address = "localhost:7075";
    let stream = TcpStream::connect(&address).await.unwrap();

    let mut peer_handler = PeerHandler::new(Network::Live, address, stream);
    peer_handler.handle().await.unwrap();
}

struct PeerHandler {
    network: Network,
    address: String,
    stream: TcpStream,
}

impl PeerHandler {
    fn new(network: Network, address: &str, stream: TcpStream) -> Self {
        Self {
            network,
            address: address.into(),
            stream,
        }
    }

    async fn handle(&mut self) -> anyhow::Result<()> {
        self.query_handshake().await?;

        let mut buffer = [0; 8];
        let result = self.stream.read_exact(&mut buffer).await.unwrap();
        dbg!(result);
        dbg!(buffer);

        let mut buffer = [0; 32];
        let size = self.stream.read_exact(&mut buffer).await.unwrap();
        // dbg!(query_hash);
        dbg!(buffer);

        // Assuming this is a NodeIdHandshake

        // let mut buffer = [0; Header::LENGTH];
        // let result = r.read_exact(&mut buffer).await.unwrap();
        // dbg!(&result);
        // let header = Header::deserialize(Network::Live, &buffer).unwrap();
        // dbg!(&header);
        todo!()
    }

    async fn query_handshake(&mut self) -> anyhow::Result<()> {
        let header = Header::new(
            Network::Live,
            MessageType::NodeIdHandshake,
            *Flags::new().set_query(true),
        );
        self.stream.write_all(&header.serialize()).await?;

        let payload = [0u8; 32];
        self.stream.write_all(&payload).await?;

        Ok(())
    }
}
