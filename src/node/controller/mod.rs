mod blocks;
mod command;
mod genesis;
mod messages;

use crate::blocks::Block;
use crate::encoding::to_hex;
use crate::network::Network;
pub(crate) use crate::node::controller::command::{
    ControllerCommand, ControllerMessageReceiver, ControllerMessageSender,
};
use crate::node::header::{Extensions, Header, MessageType};
use crate::node::messages::frontier_resp::FrontierResp;
use crate::node::state::ArcState;
use crate::node::wire::Wire;
use crate::{Public, Rai};
use anyhow::{anyhow, Context};
use futures::future::OptionFuture;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, instrument, trace};

/// A message sent between channels that contains a peer's network data.
#[derive(Debug)]
pub struct Packet {
    /// Used by pcap to annotate direction and packet number, etc.
    pub annotation: Option<String>,

    /// The data sent to/from a peer.
    pub data: Vec<u8>,
}

impl Packet {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            annotation: None,
        }
    }

    pub fn new_with_annotation(data: Vec<u8>, annotation: String) -> Self {
        Self {
            data,
            annotation: Some(annotation),
        }
    }
}

enum RecvState {
    /// Waiting for a header.
    Header,

    /// Waiting for the payload.
    Payload(Header),
}

/// The controller handles the logic of one peer. It handles and emits messages, as well as time
/// based actions, peer management, etc.
pub struct Controller {
    /// Disable when used for pcap dump, where might have our own different cookie.
    pub validate_handshakes: bool,

    network: Network,
    state: ArcState,
    peer_addr: SocketAddr,
    recv_state: RecvState,

    /// Are we doing a frontier req stream? (Bootstrap?)
    frontier_stream: bool,

    /// Internal buffer for incoming data.
    incoming_buffer: Vec<u8>,

    /// Incoming data from the connected peer.
    peer_rx: mpsc::Receiver<Packet>,

    /// Data to be sent to the other peer.
    peer_tx: mpsc::Sender<Packet>,

    /// Incoming RPC commands.
    rpc_rx: ControllerMessageReceiver,

    /// A reusable header to reduce allocations.
    // pub(crate) header: Header,
    last_annotation: Option<String>,
}

impl Controller {
    pub fn new_with_channels(
        network: Network,
        state: ArcState,
        peer_addr: SocketAddr,
    ) -> (
        Self,
        mpsc::Sender<Packet>,
        mpsc::Receiver<Packet>,
        ControllerMessageSender,
    ) {
        // Packets coming in from a remote host.
        let (incoming_tx, incoming_rx) = mpsc::channel::<Packet>(100);
        // Packets to be sent out to a remote host.
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<Packet>(100);
        // Controller RPC Commands
        let (rpc_tx, rpc_rx) = mpsc::channel::<ControllerCommand>(100);

        let s = Self {
            validate_handshakes: true,
            network,
            state,
            peer_addr,
            recv_state: RecvState::Header,
            frontier_stream: false,
            incoming_buffer: Vec::with_capacity(10_000),
            peer_rx: incoming_rx,
            peer_tx: outgoing_tx,
            rpc_rx,
            last_annotation: None,
        };

        (s, incoming_tx, outgoing_rx, rpc_tx)
    }

    /// Run will loop forever and is expected to be spawned and will quit when the incoming channel
    /// is closed.
    pub async fn run(mut self) -> anyhow::Result<()> {
        // macro_rules! handle {
        //     ($self: ident, $fun:ident, $header:expr) => {{
        //         let sh = Some(&$header);
        //         let payload = self
        //             .recv(sh)
        //             .await
        //             .with_context(|| format!("Receiving payload for {:?}", $header))?;
        //
        //         match &self.last_annotation {
        //             Some(a) => debug!("{} {:?}", a, &payload),
        //             None => debug!("{:?}", &payload),
        //         };
        //
        //         $self
        //             .$fun(&$header, payload)
        //             .await
        //             .with_context(|| format!("Handling payload for {:?}", $header))?;
        //     };};
        // }

        trace!("Initial handshake");
        self.send_handshake().await?;
        // trace!("Initial telemetry request");
        // self.send_telemetry_req().await?;

        loop {
            tokio::select! {
                maybe_packet = self.peer_rx.recv() => {
                    match maybe_packet {
                        Some(packet) => self.handle_packet(packet).await?,
                        None => todo!("Peer disconnected"),
                    }
                }
                message = self.rpc_rx.recv() => {
                    dbg!(message);
                }
            }

            // if self.frontier_stream {
            //     let payload = self.recv::<FrontierResp>(None).await?;
            //     self.handle_frontier_resp(payload).await?;
            // } else {
            //     let header = self.recv::<Header>(None).await?;
            //     header.validate(&self.network)?;
            //
            //     match header.message_type() {
            //         MessageType::Keepalive => handle!(self, handle_keepalive, header),
            //         MessageType::Publish => handle!(self, handle_publish, header),
            //         MessageType::ConfirmReq => handle!(self, handle_confirm_req, header),
            //         MessageType::ConfirmAck => handle!(self, handle_confirm_ack, header),
            //         MessageType::FrontierReq => handle!(self, handle_frontier_req, header),
            //         MessageType::Handshake => handle!(self, handle_handshake, header),
            //         MessageType::TelemetryReq => handle!(self, handle_telemetry_req, header),
            //         MessageType::TelemetryAck => handle!(self, handle_telemetry_ack, header),
            //         // MessageType::BulkPull => {}
            //         // MessageType::BulkPush => {}
            //         // MessageType::BulkPullAccount => {}
            //         _ => panic!("{:?}", header),
            //     };
            // }
        }
    }

    async fn handle_packet(&mut self, packet: Packet) -> anyhow::Result<()> {
        if let Some(annotation) = packet.annotation {
            self.last_annotation = Some(annotation);
        }
        self.incoming_buffer.extend(packet.data);

        match self.recv_state {
            RecvState::Header => {
                if let Some(header) = self.recv::<Header>(None)? {
                    self.recv_state = RecvState::Payload(header);
                }
            }
            RecvState::Payload(header) => {
                todo!()
            }
        }
        Ok(())
    }

    #[instrument(skip(self, header))]
    fn recv<T: Wire + Debug>(&mut self, header: Option<&Header>) -> anyhow::Result<Option<T>> {
        let bytes = T::len(header)?;
        if self.incoming_buffer.len() < bytes {
            // Not enough bytes for T
            return Ok(None);
        }

        let buffer = self.incoming_buffer[0..bytes].to_owned();
        self.incoming_buffer = Vec::from(&self.incoming_buffer[bytes..]);
        trace!("HEX: {}", to_hex(&buffer));
        let result = T::deserialize(header, &buffer)?;
        Ok(Some(result))
    }

    // async fn recv_buf(&mut self, size: usize) -> anyhow::Result<Vec<u8>> {
    //     // TODO: Idle timeout so a toxic node can't just leave empty connections running without
    //     //       any traffic.
    //     loop {
    //         if self.incoming_buffer.len() >= size {
    //             return self.recv_immediate(size);
    //         }
    //
    //         let packet = match self.peer_rx.recv().await {
    //             Some(data) => data,
    //             None => {
    //                 return Err(anyhow!(
    //                     "Incoming stream disconnected {:?} {:?}",
    //                     self.peer_addr,
    //                     self.last_annotation
    //                 ))
    //             }
    //         };
    //
    //         if let Some(annotation) = packet.annotation {
    //             self.last_annotation = Some(annotation);
    //         }
    //         self.incoming_buffer.extend(packet.data);
    //     }
    // }
    //
    fn recv_immediate(&mut self, size: usize) -> anyhow::Result<Vec<u8>> {
        debug_assert!(self.incoming_buffer.len() >= size);

        // This is super inefficient. Need to use something like
        // https://crates.io/crates/slice-deque
        // Might not work in wasm later.

        let buf = self.incoming_buffer[0..size].to_owned();
        self.incoming_buffer = Vec::from(&self.incoming_buffer[size..]);
        Ok(buf)
    }

    #[instrument(level = "debug", skip(self, message))]
    async fn send<T: Wire + Debug>(&mut self, message: &T) -> anyhow::Result<()> {
        let data = message.serialize();
        trace!("HEX {}", to_hex(&data));
        debug!("OBJ {:?}", &message);
        self.peer_tx.send(Packet::new(Vec::from(data))).await?;
        Ok(())
    }

    async fn send_header(
        &mut self,
        message_type: MessageType,
        ext: Extensions,
    ) -> anyhow::Result<()> {
        let header = Header::new(self.network, message_type, ext);
        Ok(self.send(&header).await?)
    }

    /// Set up the genesis block if it hasn't already.
    pub async fn init(&mut self) -> anyhow::Result<()> {
        self.ensure_genesis().await.context("Ensuring genesis")?;
        Ok(())
    }

    /// Update the representative weights based on this block being added to the network.
    pub async fn balance_rep_weights(&mut self, _full_block: &Block) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn account_balance(&self, account: &Public) -> anyhow::Result<Rai> {
        let context = || anyhow!("Account balance for {:?}", account);
        let block = self.get_latest_block(account).await.with_context(context)?;

        match block {
            Some(block) => Ok(block.balance().to_owned()),
            None => Ok(Rai::zero()),
        }
    }

    pub fn network(&self) -> &Network {
        &self.network
    }

    pub fn peer_addr(&self) -> &SocketAddr {
        &self.peer_addr
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4};
    use std::str::FromStr;
    use std::sync::Arc;

    use tokio::sync::Mutex;

    use crate::blocks::{Block, BlockHash, OpenBlock, Previous, SendBlock};
    use crate::network::DEFAULT_PORT;
    use crate::node::state::MemoryState;
    use crate::Address;

    use super::*;

    async fn empty_lattice(network: Network) -> Controller {
        let state = Arc::new(Mutex::new(MemoryState::new(network)));
        let (mut controller, _rx, _tx) = Controller::new_with_channels(
            network,
            state,
            None,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_PORT)),
        );
        controller.init().await.unwrap();
        controller
    }

    #[tokio::test]
    async fn genesis() {
        let network = Network::Live;
        let genesis = network.genesis_block();

        let controller = empty_lattice(network).await;
        assert_eq!(
            controller
                .get_latest_block(genesis.account())
                .await
                .unwrap()
                .unwrap()
                .balance(),
            &Rai::max()
        );
    }

    /// Genesis Account: genesis (Open) -> gen_send (Send)
    /// Landing Account:                -> land_open (Open) -> land_send (Send)
    #[tokio::test]
    async fn send_then_recv_to_new_account() {
        let network = Network::Live;
        let genesis = network.genesis_block();

        let landing_account =
            Address::from_str("nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo")
                .unwrap()
                .to_public();

        let mut controller = empty_lattice(network).await;

        let gen_send: SendBlock = serde_json::from_str(
            r#"{
                "type": "send",
                "previous": "991CF190094C00F0B68E2E5F75F6BEE95A2E0BD93CEAA4A6734DB9F19B728948",
                "destination": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "balance": "FD89D89D89D89D89D89D89D89D89D89D",
                "work": "3c82cc724905ee95",
                "signature": "5B11B17DB9C8FE0CC58CAC6A6EECEF9CB122DA8A81C6D3DB1B5EE3AB065AA8F8CB1D6765C8EB91B58530C5FF5987AD95E6D34BB57F44257E20795EE412E61600"
            }"#,
        )
        .unwrap();

        // TODO: This should be done somewhere (the controller?
        // e.g. controller.validate_send_block() or controller.fill_send_block()
        let block: Block =
            Block::from_send_block(&gen_send, genesis.account(), genesis.representative());

        controller.add_elected_block(&block).await.unwrap();

        let given = Rai::from(3271945835778254456378601994536232802u128);

        let genesis_balance = Rai::max().checked_sub(&given).unwrap();

        // The genesis account has a reduced amount because they've created a send block.
        assert_eq!(
            controller
                .account_balance(&genesis.account())
                .await
                .unwrap(),
            genesis_balance
        );

        // Account isn't opened yet so it's empty.
        assert_eq!(
            controller.account_balance(&landing_account).await.unwrap(),
            Rai::zero()
        );

        // TODO: Check pending balance of landing account.

        // A real open block to the "Landing" account.
        // `type` is ignored here, but just left it in as it's part of the RPC response and
        // might be checked in the future.
        let land_open: OpenBlock = serde_json::from_str(
            r#"{
                "type": "open",
                "source": "A170D51B94E00371ACE76E35AC81DC9405D5D04D4CEBC399AEACE07AE05DD293",
                "representative": "nano_1awsn43we17c1oshdru4azeqjz9wii41dy8npubm4rg11so7dx3jtqgoeahy",
                "account": "nano_13ezf4od79h1tgj9aiu4djzcmmguendtjfuhwfukhuucboua8cpoihmh8byo",
                "work": "e997c097a452a1b1",
                "signature": "E950FFDF0C9C4DAF43C27AE3993378E4D8AD6FA591C24497C53E07A3BC80468539B0A467992A916F0DDA6F267AD764A3C1A5BDBD8F489DFAE8175EEE0E337402"
            }"#,
        ).unwrap();
        let land_open = Block::from_open_block(&land_open, &Previous::Open, &given);
        assert_eq!(
            land_open.hash().unwrap(),
            &BlockHash::from_str(
                "90D0C16AC92DD35814E84BFBCC739A039615D0A42A76EF44ADAEF1D99E9F8A35"
            )
            .unwrap()
        );

        controller.add_elected_block(&land_open).await.unwrap();

        assert_eq!(
            controller.account_balance(&landing_account).await.unwrap(),
            given
        );

        let land_send: SendBlock = serde_json::from_str(
            r#"{
    "type": "send",
    "previous": "90D0C16AC92DD35814E84BFBCC739A039615D0A42A76EF44ADAEF1D99E9F8A35",
    "destination": "nano_35jjmmmh81kydepzeuf9oec8hzkay7msr6yxagzxpcht7thwa5bus5tomgz9",
    "balance": "02761762762762762762762762762762",
    "work": "6d6d59ca60cab77d",
    "signature": "434CF7E7B2C2CAA3E3910CC711B29498870636C1247EA8C72BD5C0A7BB15A7BACFEC9CF289B92E4BD56F56E68277B45B3A3FF9339D2547038B87DE38C851B70B"
  }"#).unwrap();

        let land_send =
            Block::from_send_block(&land_send, &landing_account, &land_open.representative());

        controller.add_elected_block(&land_send).await.unwrap();

        assert_eq!(
            controller.account_balance(&landing_account).await.unwrap(),
            given
                .checked_sub(&Rai::from(324518553658426726783156020576256))
                .unwrap()
        );
    }
}
