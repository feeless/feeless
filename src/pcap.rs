use crate::network::Network;
use crate::network::DEFAULT_PORT;
use crate::node::{MemoryState, Packet, Peer};
use anyhow::Context;
use chrono::{DateTime, Utc};
use etherparse::{InternetSlice, SlicedPacket};
use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice, TransportSlice};
use pcarp::Capture;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tracing::{debug, error, info, trace, warn};

/// Subject is the focused peer that we act as "us", when showing if we're sending or
/// receiving.
//
// TODO: Infer peers or by valid header?
// Might be nicer if it can learn peers from the dump in case there are other port used.
// Another option is to just parse every packet and if the header is not valid, just
// ignore it.
// Or... just assume the first packet sent is from the subject.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Subject {
    AutoFirstSource,
    Specified(Ipv4Addr),
}

enum Direction {
    Send,
    Recv,
}

pub(crate) struct PcapDump {
    /// Storage to continue a TCP payload for the next packet in a stream.
    stream_cont: HashMap<String, (usize, Vec<u8>)>,

    /// Frontier connections
    frontiers: HashSet<String>,

    /// per_stream_peers
    peers: HashMap<String, Sender<Packet>>,

    pub start_at: Option<usize>,
    pub end_at: Option<usize>,
    pub filter_addr: Option<Ipv4Addr>,

    subject: Subject,
    found_subject: Option<Ipv4Addr>,

    packet_idx: usize,
    stream_id: String,
}

impl PcapDump {
    pub fn new(subject: Subject) -> Self {
        let found_subject = match subject {
            Subject::Specified(s) => Some(s),
            _ => None,
        };

        PcapDump {
            stream_cont: HashMap::new(),
            frontiers: HashSet::new(),
            subject,
            found_subject,
            packet_idx: 0,
            stream_id: "".to_string(),
            start_at: None,
            end_at: None,
            filter_addr: None,
            peers: Default::default(),
        }
    }

    pub async fn dump(&mut self, path: &str) -> anyhow::Result<()> {
        let network = Network::Live;
        let state = Arc::new(Mutex::new(MemoryState::new(network)));

        info!("Loading dump: {}", path);

        let file = File::open(path).with_context(|| format!("Opening file {}", path))?;

        let mut has_started = false;
        let mut reader =
            Capture::new(file).with_context(|| format!("Reading capture file {:?}", &path))?;
        self.packet_idx = 0;
        'next_packet: loop {
            self.packet_idx += 1; // 1 based packet numbering because wireshark uses it.

            let packet = reader
                .next()
                .transpose()
                .with_context(|| format!("Reading next packet: {}", self.packet_idx))?;
            let packet = if packet.is_none() {
                // EOF
                debug!("No more packets in pcap. Waiting for cleanup, then exiting.");
                // TODO: Do this a better way, maybe give the peer an internal only exit message.
                tokio::time::sleep(Duration::from_secs(1)).await;
                return Ok(());
            } else {
                packet.unwrap()
            };
            let timestamp: DateTime<Utc> = packet.timestamp.unwrap().into();
            let packet = match SlicedPacket::from_ethernet(&packet.data).with_context(|| {
                format!(
                    "Parsing packet data to ethernet for packet {}",
                    self.packet_idx
                )
            }) {
                Ok(p) => p,
                Err(err) => {
                    warn!("Packet was no parsed correctly because: {:?}", err);
                    continue 'next_packet;
                }
            };
            let (ip, tcp, data) = match Self::process_packet(&packet) {
                Some(r) => r,
                None => continue,
            };

            // Work out direction based on subject
            if self.subject == Subject::AutoFirstSource && self.found_subject.is_none() {
                self.found_subject = Some(ip.source_addr());
            }
            let subject = self.found_subject.expect("a subject to be set by now");
            let direction = if ip.destination_addr() == subject {
                Direction::Recv
            } else if ip.source_addr() == subject {
                Direction::Send
            } else {
                warn!("Unknown direction for {} and {:?}", subject, ip);
                Direction::Recv
            };

            // Start and end packet happens after the subject code, so we can still use the
            // first source from the first packet.
            if !has_started {
                match self.start_at {
                    Some(start_at) => {
                        if start_at <= self.packet_idx {
                            has_started = true;
                        } else {
                            continue;
                        }
                    }
                    None => has_started = true,
                }
            }
            if let Some(end_at) = self.end_at {
                if self.packet_idx > end_at {
                    return Ok(());
                }
            }

            if data.len() == 0 {
                continue;
            }

            // Only look at port 7075.
            if tcp.destination_port() != DEFAULT_PORT && tcp.source_port() != DEFAULT_PORT {
                continue;
            }

            if let Some(addr) = self.filter_addr {
                if ip.source_addr() != addr && ip.destination_addr() != addr {
                    continue;
                }
            }

            self.stream_id = format!(
                "{}:{}->{}:{}",
                ip.source_addr(),
                tcp.source_port(),
                ip.destination_addr(),
                tcp.destination_port()
            );

            let mut connection_id = vec![
                ip.source_addr().to_string(),
                tcp.source_port().to_string(),
                ip.destination_addr().to_string(),
                tcp.destination_port().to_string(),
            ];
            connection_id.sort();
            let connection_id = connection_id.join("-");

            let direction_text = match direction {
                Direction::Send => {
                    format!(">>> {}:{}", ip.destination_addr(), tcp.destination_port())
                }
                Direction::Recv => format!("<<< {}:{}", ip.source_addr(), tcp.source_port()),
            };

            let annotation = format!(
                "Packet: #{} {} {} size: {}",
                &self.packet_idx,
                timestamp.format("%+"),
                direction_text,
                data.len(),
            );

            let tx = match self.peers.get(&connection_id) {
                Some(z) => z,
                None => {
                    let state_cloned = state.clone();
                    let peer_addr =
                        SocketAddr::new(IpAddr::V4(ip.destination_addr()), tcp.destination_port());
                    let (mut peer, tx, mut rx) =
                        Peer::new_with_channels(network, state_cloned, peer_addr.clone());

                    // Discard all responses from the peer since we are just processing
                    // packets.
                    tokio::spawn(async move {
                        loop {
                            if rx.recv().await.is_none() {
                                trace!("Receiving channel has closed.");
                                return;
                            }
                        }
                    });

                    tokio::spawn(async move {
                        peer.validate_handshakes = false;
                        let result = peer.run().await;
                        if let Err(err) = result {
                            error!("Error on pcap controller {:?}: {:?}", peer_addr, err);
                        }
                    });

                    self.peers.insert(connection_id.clone(), tx);
                    self.peers.get(&connection_id).unwrap()
                }
            };

            tx.send(Packet::new_with_annotation(Vec::from(data), annotation))
                .await?;
        }
    }

    fn process_packet<'p>(
        packet: &'p SlicedPacket,
    ) -> Option<(&'p Ipv4HeaderSlice<'p>, &'p TcpHeaderSlice<'p>, &'p [u8])> {
        // TODO: Support IPv6
        let ip = if let Some(InternetSlice::Ipv4(ip)) = &packet.ip {
            ip
        } else {
            return None;
        };

        let tcp = if let Some(TransportSlice::Tcp(tcp)) = &packet.transport {
            tcp
        } else {
            return None;
        };

        let data_len = ip.payload_len() as usize - tcp.slice().len() as usize;
        Some((ip, tcp, &packet.payload[..data_len]))
    }
}
