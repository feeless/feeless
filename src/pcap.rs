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
use anyhow::anyhow;
use etherparse::SlicedPacket;
use etherparse::TransportSlice;
use pcap_parser::traits::PcapReaderIterator;
use pcap_parser::{Block, PcapBlockOwned, PcapError, PcapNGReader};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tracing::{debug, info, trace, warn};

enum Direction {
    Send,
    Recv,
}

impl Direction {
    fn swap(&mut self) {
        *self = match self {
            Direction::Send => Direction::Recv,
            Direction::Recv => Direction::Send,
        };
    }
}

pub async fn pcap_dump(path: &str) -> anyhow::Result<()> {
    info!("Loading dump: {}", path);

    let mut direction = Direction::Send;
    let file = File::open(path)?;

    let recv_color = Color::Green.normal();
    let send_color = Color::Blue.bold();
    let direction_marker_color = Color::Yellow.bold();
    let error_color = Color::Red;

    let mut reader = PcapNGReader::new(65536, file)?;
    let mut packet_idx = 0;
    'packet: loop {
        packet_idx += 1; // 1 based packet numbering because wireshark uses it.
        let data = next_packet(&mut reader)?;
        let data = if data.is_none() {
            // EOF
            return Ok(());
        } else {
            data.unwrap()
        };

        let packet = SlicedPacket::from_ethernet(&data)?;
        let tcp = if let Some(TransportSlice::Tcp(tcp)) = &packet.transport {
            tcp
        } else {
            continue;
        };

        // Only look at port 7075.
        // TODO: Infer peers or by valid header?
        // Might be nicer if it can learn peers from the dump in case there are other port used.
        // Another option is to just parse every packet and if the header is not valid, just
        // ignore it.
        if tcp.destination_port() != DEFAULT_PORT && tcp.source_port() != DEFAULT_PORT {
            continue;
        }

        let bytes = packet.payload;

        // TODO: WTF: packet.payload is giving two extra bytes at the end of every packet.
        // let bytes = &bytes[0..bytes.len() - 2];

        trace!("packet: {} size: {}", &packet_idx, bytes.len());
        trace!("dump: {}", to_hex(&bytes));

        let mut bytes = Bytes::new(bytes);
        while !bytes.eof() {
            dbg!(bytes.remain());
            let header = Header::deserialize(None, bytes.slice(Header::LEN)?)?;
            let h = Some(&header);
            let (direction_text, color) = match direction {
                Direction::Send => (">>>", send_color),
                Direction::Recv => ("<<<", recv_color),
            };

            dbg!(&header);

            let func = match header.message_type() {
                MessageType::Handshake => payload::<Handshake>,
                MessageType::ConfirmReq => payload::<ConfirmReq>,
                MessageType::ConfirmAck => payload::<ConfirmAck>,
                MessageType::Keepalive => payload::<Keepalive>,
                MessageType::TelemetryReq => payload::<TelemetryReq>,
                MessageType::TelemetryAck => payload::<TelemetryAck>,
                MessageType::Publish => payload::<Publish>,
                m => {
                    println!("{}", error_color.paint(format!("TODO {:?}", header)));
                    warn!("Aborting packet!");
                    continue 'packet;
                }
            };
            dbg!(bytes.remain());
            let p = func(h, &mut bytes)?;
            dbg!(bytes.remain());
            let dbg = format!("{:#?}", p.as_ref());
            println!(
                "{} {}",
                direction_marker_color.paint(direction_text),
                color.paint(dbg)
            );
        }

        // TODO: This doesn't apply now that we're doing pcap capture. It should know the IP
        // address of the network, and use that as the source. Maybe scan through it and work out
        // the most used one.
        //
        // direction.swap();
    }
}

/// Returns `Ok(None)` when EOF
// TODO: I don't know how to return a reference slice. Lifetime problems.
fn next_packet(reader: &mut PcapNGReader<File>) -> anyhow::Result<Option<Vec<u8>>> {
    loop {
        let result = &reader.next();
        let (offset, block) = match result {
            Ok(ok) => ok,
            Err(err) => {
                return match err {
                    PcapError::Eof => Ok(None),
                    err => Err(anyhow!("Pcap error: {:?}", err)),
                };
            }
        };
        let ng = match block {
            PcapBlockOwned::NG(ng) => ng,
            _ => return Err(anyhow!("only ng blocks supported")),
        };

        let data = match ng {
            Block::EnhancedPacket(ep) => ep.data,
            Block::SimplePacket(sp) => sp.data,
            _ => {
                // Ignoring non packet data.
                reader.consume(*offset);
                continue;
            }
        };

        let data = data.to_owned();
        reader.consume(*offset);
        return Ok(Some(data));
    }
}

pub fn payload<T: 'static + Wire>(
    header: Option<&Header>,
    bytes: &mut Bytes,
) -> anyhow::Result<Box<dyn Wire>> {
    let len = T::len(header)?;

    if bytes.remain() < len {
        return Err(anyhow!(
            "not enough bytes left to process. want: {}, has: {}",
            len,
            bytes.remain()
        ));
    }

    let data = bytes.slice(len)?;
    let payload: T = T::deserialize(header, data)?;
    Ok(Box::new(payload))
}
