use crate::bytes::Bytes;
use crate::node::header::{Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;

use crate::node::messages::frontier_req::FrontierReq;
use crate::node::messages::frontier_resp::FrontierResp;
use crate::node::messages::handshake::Handshake;
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::messages::telemetry_ack::TelemetryAck;
use crate::node::messages::telemetry_req::TelemetryReq;
use crate::node::wire::Wire;
use crate::{to_hex, DEFAULT_PORT};
use ansi_term;
use ansi_term::Color;
use anyhow::{anyhow, Context};
use etherparse::{InternetSlice, SlicedPacket};
use etherparse::{Ipv4HeaderSlice, TcpHeaderSlice, TransportSlice};
use pcarp::Capture;
use std::collections::{HashMap, HashSet};

use std::fs::File;
use std::io::Read;
use std::net::Ipv4Addr;

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
pub enum Subject {
    AutoFirstSource,
    Specified(Ipv4Addr),
}

enum Direction {
    Send,
    Recv,
}

pub struct PcapDump<'a> {
    /// Storage to continue a TCP payload for the next packet in a stream.
    stream_cont: HashMap<String, (usize, Vec<u8>)>,

    /// Frontier connections
    frontiers: HashSet<String>,

    pub expanded: bool,
    pub start_at: Option<usize>,
    pub end_at: Option<usize>,
    pub filter_addr: Option<Ipv4Addr>,
    pub abort_on_error: bool,
    pub pause_on_error: bool,

    subject: Subject,
    found_subject: Option<Ipv4Addr>,

    packet_idx: usize,
    stream_id: String,
    bytes: Bytes<'a>,
}

impl<'a> PcapDump<'a> {
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
            expanded: false,
            start_at: None,
            end_at: None,
            filter_addr: None,
            abort_on_error: false,
            pause_on_error: false,
            bytes: Bytes::new(&[]),
        }
    }

    pub fn dump(&'a mut self, path: &'a str) -> anyhow::Result<()> {
        info!("Loading dump: {}", path);

        let file = File::open(path).with_context(|| format!("Opening file {}", path))?;

        let recv_color = Color::Green.normal();
        let send_color = Color::Blue.bold();
        let direction_marker_color = Color::White.bold();
        let _error_color = Color::Red;

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
                return Ok(());
            } else {
                packet.unwrap()
            };
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

            debug!(
                "Packet: #{} size: {} {}",
                &self.packet_idx,
                data.len(),
                &self.stream_id
            );

            let mut v = vec![];
            let slice = match self.stream_cont.get(&self.stream_id) {
                Some((other_packet_idx, b)) => {
                    // We have some left over data from a previous packet.
                    trace!(
                        "Prepending {} bytes from packet #{}.",
                        b.len(),
                        other_packet_idx
                    );
                    v.extend_from_slice(&b);
                    v.extend_from_slice(data);
                    self.stream_cont.remove(&self.stream_id);
                    v.as_slice()
                }
                None => {
                    trace!("Payload: {}", to_hex(data));
                    data
                }
            };

            let mut bytes = Bytes::new(slice);
            if self.frontiers.contains(&connection_id) {
                // At this point we're only going to receive frontier messages which do not have a
                // header.
                while !bytes.eof() {
                    if self.should_resume_stream_later(&mut bytes, FrontierResp::LEN)? {
                        continue 'next_packet;
                    }

                    let frontier =
                        FrontierResp::deserialize(None, bytes.slice(FrontierResp::LEN)?)?;
                    // dbg!(frontier);
                }
                continue 'next_packet;
            }

            while !bytes.eof() {
                if self
                    .should_resume_stream_later(&mut bytes, Header::LEN)
                    .context("Header")?
                {
                    continue 'next_packet;
                }

                let header_bytes =
                    match self.handle_error(bytes.slice(Header::LEN).with_context(|| {
                        format!("Slicing header on packet #{}", self.packet_idx)
                    }))? {
                        Some(x) => x,
                        None => continue 'next_packet,
                    };

                let header = match self.handle_error(
                    Header::deserialize(None, header_bytes).with_context(|| {
                        format!("Deserializing header on packet #{}", self.packet_idx)
                    }),
                )? {
                    Some(x) => x,
                    None => continue 'next_packet,
                };

                let (direction_text, color) = match direction {
                    Direction::Send => (
                        format!(
                            ">>> #{} {}:{}",
                            self.packet_idx,
                            ip.destination_addr(),
                            tcp.destination_port()
                        ),
                        send_color,
                    ),
                    Direction::Recv => (
                        format!(
                            "<<< #{} {}:{}",
                            self.packet_idx,
                            ip.source_addr(),
                            tcp.source_port()
                        ),
                        recv_color,
                    ),
                };

                let func = match header.message_type() {
                    MessageType::Handshake => payload::<Handshake>,
                    MessageType::ConfirmReq => payload::<ConfirmReq>,
                    MessageType::ConfirmAck => payload::<ConfirmAck>,
                    MessageType::Keepalive => payload::<Keepalive>,
                    MessageType::TelemetryReq => payload::<TelemetryReq>,
                    MessageType::TelemetryAck => payload::<TelemetryAck>,
                    MessageType::Publish => payload::<Publish>,
                    MessageType::FrontierReq => payload::<FrontierReq>,
                    _ => {
                        let o = self.handle_error::<anyhow::Result<()>>(Err(anyhow!(
                            "Unhandled message type {:?}",
                            header
                        )))?;
                        debug_assert!(o.is_none());
                        continue 'next_packet;
                    }
                };

                let maybe_decoded = match self.handle_error(
                    func(Some(&header), &mut bytes)
                        .with_context(|| format!("Decoding packet #{}", &self.packet_idx)),
                )? {
                    Some(x) => x,
                    None => continue 'next_packet,
                };
                let decoded = match maybe_decoded {
                    Some(p) => p,
                    None => {
                        bytes.seek(-(Header::LEN as i64))?;
                        self.save_stream_for_later(&mut bytes).context("Header")?;
                        continue 'next_packet;
                    }
                };

                let dbg = if self.expanded {
                    format!("{:#?}", decoded.as_ref())
                } else {
                    format!("{:?}", decoded.as_ref())
                };
                println!(
                    "{} {}",
                    direction_marker_color.paint(direction_text),
                    color.paint(dbg)
                );

                if header.message_type() == MessageType::FrontierReq {
                    self.frontiers.insert(connection_id.clone());
                }
            }
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

    /// If theres an error, we either return the error, or optionally allow the user to continue.
    /// Returning Ok(None) tells the caller the packet is bad and we want to try the next packet.
    fn handle_error<T>(
        &self,
        result: Result<T, anyhow::Error>,
    ) -> Result<Option<T>, anyhow::Error> {
        match result {
            Ok(m) => Ok(Some(m)),
            Err(err) => {
                if self.abort_on_error {
                    return Err(err);
                }

                error!("{:?}", err);
                if self.pause_on_error {
                    println!("\nPress [enter] to resume");
                    std::io::stdin().read(&mut [0]).unwrap();
                }
                Ok(None)
            }
        }
    }

    fn should_resume_stream_later(
        &mut self,
        bytes: &mut Bytes,
        expected_len: usize,
    ) -> anyhow::Result<bool> {
        if bytes.remain() < expected_len {
            self.save_stream_for_later(bytes)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn save_stream_for_later(&mut self, bytes: &mut Bytes) -> anyhow::Result<()> {
        let remaining = Vec::from(
            bytes
                .slice(bytes.remain())
                .context("Slicing remaining stream for later")?,
        );
        self.stream_cont
            .insert(self.stream_id.to_owned(), (self.packet_idx, remaining));
        Ok(())
    }
}

pub fn payload<T: 'static + Wire>(
    header: Option<&Header>,
    bytes: &mut Bytes,
) -> anyhow::Result<Option<Box<dyn Wire>>> {
    let len = T::len(header)?;

    if bytes.remain() < len {
        trace!(
            "Not enough bytes left to process. Needs {} more. Will prepend {} bytes in next packet.",
            len - bytes.remain(),
            bytes.remain()
        );
        return Ok(None);
    }

    let data = bytes.slice(len)?;
    let payload: T = T::deserialize(header, data).context("Deserializing payload")?;
    Ok(Some(Box::new(payload)))
}
