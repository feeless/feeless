use crate::bytes::Bytes;
use crate::node::header::{Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;
use crate::node::messages::empty::Empty;
use crate::node::messages::handshake::Handshake;
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::wire::Wire;

use crate::node::messages::telemetry_ack::TelemetryAck;
use crate::node::messages::telemetry_req::TelemetryReq;
use crate::{to_hex, DEFAULT_PORT};
use ansi_term;
use ansi_term::Color;
use anyhow::{anyhow, Context, Error};
use etherparse::TransportSlice;
use etherparse::{InternetSlice, SlicedPacket};
use pcarp::Capture;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::str::FromStr;
use tracing::{debug, error, info, trace, warn};

#[derive(Debug)]
pub enum Subject {
    AutoFirstSource,
    AutoMostSeen,
    Specified(IpAddr),
}

enum Direction {
    Send,
    Recv,
}

pub struct PcapDump {
    /// Storage to continue a TCP payload for the next packet in a stream.
    stream_cont: HashMap<String, Vec<u8>>,

    /// Subject is the focused peer that we act as "us", when showing if we're sending or
    /// receiving.
    subject: Subject,
}

impl PcapDump {
    pub fn new(subject: Subject) -> Self {
        PcapDump {
            stream_cont: HashMap::new(),
            subject,
        }
    }

    pub fn dump(&mut self, path: &str) -> anyhow::Result<()> {
        let subject = match self.subject {
            Subject::Specified(s) => s,
            _ => {
                // TODO: Support automatic subject detection.
                return Err(anyhow!("{:?} not supported yet!", self.subject));
            }
        };
        if !subject.is_ipv4() {
            // TODO: ipv6
            return Err(anyhow!("specific source only supports ipv4"));
        }

        info!("Loading dump: {}", path);

        let file = File::open(path)?;

        let recv_color = Color::Green.normal();
        let send_color = Color::Blue.bold();
        let direction_marker_color = Color::White.bold();
        let error_color = Color::Red;

        let mut reader = Capture::new(file)?;
        let mut packet_idx = 0;
        'next_packet: loop {
            packet_idx += 1; // 1 based packet numbering because wireshark uses it.
            let data = reader.next().transpose()?;
            let packet = if data.is_none() {
                // EOF
                return Ok(());
            } else {
                data.unwrap()
            };
            let data = packet.data;

            let packet = SlicedPacket::from_ethernet(&data)?;
            // TODO: Support IPv6
            let ip = if let Some(InternetSlice::Ipv4(ip)) = &packet.ip {
                ip
            } else {
                continue;
            };

            // Direction
            // TODO: Infer peers or by valid header?
            // Might be nicer if it can learn peers from the dump in case there are other port used.
            // Another option is to just parse every packet and if the header is not valid, just
            // ignore it.
            // Or... just assume the first packet sent is from the subject.
            let source = if let IpAddr::V4(v4_source) = subject {
                v4_source
            } else {
                continue;
            };

            direction = if ip.destination_addr() == source {
                Direction::Recv
            } else if ip.source_addr() == source {
                Direction::Send
            } else {
                warn!("Unknown direction for {} and {:?}", source, ip);
                Direction::Recv
            };

            let tcp = if let Some(TransportSlice::Tcp(tcp)) = &packet.transport {
                tcp
            } else {
                continue;
            };

            // Only look at port 7075.
            if tcp.destination_port() != DEFAULT_PORT && tcp.source_port() != DEFAULT_PORT {
                continue;
            }

            let stream_id = format!(
                "{}:{}->{}:{}",
                ip.source_addr(),
                tcp.source_port(),
                ip.destination_addr(),
                tcp.destination_port()
            );

            let mut v = vec![];
            let bytes = match self.stream_cont.get(&stream_id) {
                Some(b) => {
                    // We have some left over data from a previous packet.
                    trace!("Prepending {} bytes from a previous packet.", b.len());
                    v.extend_from_slice(&b);
                    v.extend_from_slice(&packet.payload);
                    self.stream_cont.remove(&stream_id);
                    v.as_slice()
                }
                None => packet.payload,
            };

            trace!("packet: #{} size: {}", &packet_idx, bytes.len());
            // trace!("dump: {}", to_hex(&bytes));

            let mut bytes = Bytes::new(bytes);
            while !bytes.eof() {
                let header_bytes = match bytes.slice(Header::LEN).context("slicing header") {
                    Ok(h) => h,
                    Err(err) => {
                        error!("Error processing header, skipping packet: {}", err);
                        continue 'next_packet;
                    }
                };

                let header =
                    match Header::deserialize(None, header_bytes).context("deserializing header") {
                        Ok(header) => header,
                        Err(err) => {
                            error!("Error processing header, skipping packet: {}", err);
                            continue 'next_packet;
                        }
                    };
                let (direction_text, color) = match direction {
                    Direction::Send => (format!(">>> {}", ip.destination_addr()), send_color),
                    Direction::Recv => (format!("<<< {}", ip.source_addr()), recv_color),
                };

                let func = match header.message_type() {
                    MessageType::Handshake => payload::<Handshake>,
                    MessageType::ConfirmReq => payload::<ConfirmReq>,
                    MessageType::ConfirmAck => payload::<ConfirmAck>,
                    MessageType::Keepalive => payload::<Keepalive>,
                    MessageType::TelemetryReq => payload::<TelemetryReq>,
                    // MessageType::TelemetryAck => payload::<TelemetryAck>,
                    MessageType::Publish => payload::<Publish>,
                    _ => {
                        println!("{}", error_color.paint(format!("TODO {:?}", header)));
                        continue 'next_packet;
                    }
                };
                let decoded_result = func(Some(&header), &mut bytes)
                    .with_context(|| format!("decoding packet {}", &packet_idx));
                let maybe_decoded = match decoded_result {
                    Ok(m) => m,
                    Err(err) => {
                        error!(
                            "error decoding packet payload, skipping remaining data: {}",
                            err
                        );
                        continue 'next_packet;
                    }
                };

                let decoded = match maybe_decoded {
                    Some(p) => p,
                    None => {
                        let remaining = Vec::from(bytes.slice(bytes.remain())?);
                        self.stream_cont.insert(stream_id.clone(), remaining);
                        continue 'next_packet;
                    }
                };

                // let dbg = format!("{:#?}", decoded.as_ref());
                // println!(
                //     "{} {}",
                //     direction_marker_color.paint(direction_text),
                //     color.paint(dbg)
                // );
            }
        }
    }
}

pub fn payload<T: 'static + Wire>(
    header: Option<&Header>,
    bytes: &mut Bytes,
) -> anyhow::Result<Option<Box<dyn Wire>>> {
    let len = T::len(header)?;

    if bytes.remain() < len {
        trace!(
            "Not enough bytes left to process. Will prepend {} bytes in next packet.",
            bytes.remain()
        );
        return Ok(None);
    }

    let data = bytes.slice(len)?;
    let payload: T = T::deserialize(header, data).context("deserializing payload")?;
    Ok(Some(Box::new(payload)))
}
