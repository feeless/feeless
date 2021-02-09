use crate::bytes::Bytes;
use crate::messages::confirm_ack::ConfirmAck;
use crate::messages::confirm_req::ConfirmReq;
use crate::messages::empty::Empty;
use crate::messages::handshake::{Handshake, HandshakeResponse};
use crate::messages::keepalive::Keepalive;
use crate::messages::publish::Publish;
use crate::wire::header::{Header, MessageType};
use crate::wire::Wire;
use ansi_term::Color::{Green, Yellow};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tracing::info;

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

pub async fn dump(path: &str) -> anyhow::Result<()> {
    info!("Loading dump: {}", path);

    let mut direction = Direction::Send;
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let bytes = hex::decode(line?)?;
        let mut bytes = Bytes::new(&bytes);

        while !bytes.eof() {
            let header = Header::deserialize(None, bytes.slice(Header::LEN)?)?;
            let (direction_text, color) = match direction {
                Direction::Send => (">>>", Green),
                Direction::Recv => ("<<<", Yellow),
            };
            println!(
                "{} {}",
                direction_text,
                color.paint(header.to_short_string())
            );

            let h = Some(&header);

            match header.message_type() {
                MessageType::Handshake => dump_payload::<Handshake>(h, &mut bytes)?,
                MessageType::ConfirmReq => dump_payload::<ConfirmReq>(h, &mut bytes)?,
                MessageType::ConfirmAck => dump_payload::<ConfirmAck>(h, &mut bytes)?,
                MessageType::Keepalive => dump_payload::<Keepalive>(h, &mut bytes)?,
                MessageType::TelemetryReq => dump_payload::<Empty>(h, &mut bytes)?,
                MessageType::Publish => dump_payload::<Publish>(h, &mut bytes)?,
                _ => todo!("{:?}", header),
            };
        }

        direction.swap();
    }

    Ok(())
}

pub fn dump_payload<T: Wire + Debug>(
    header: Option<&Header>,
    bytes: &mut Bytes,
) -> anyhow::Result<()> {
    let len = T::len(header)?;
    let data = bytes.slice(len)?;
    let payload: T = T::deserialize(header, data)?;
    println!("{:#?}", payload);
    Ok(())
}
