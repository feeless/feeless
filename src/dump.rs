use crate::bytes::Bytes;
use crate::node::header::{Header, MessageType};
use crate::node::messages::confirm_ack::ConfirmAck;
use crate::node::messages::confirm_req::ConfirmReq;
use crate::node::messages::empty::Empty;
use crate::node::messages::handshake::Handshake;
use crate::node::messages::keepalive::Keepalive;
use crate::node::messages::publish::Publish;
use crate::node::wire::Wire;
use ansi_term::Color::{Green, Yellow};

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
            let h = Some(&header);
            let (direction_text, color) = match direction {
                Direction::Send => (">>>", Green),
                Direction::Recv => ("<<<", Yellow),
            };

            let p = match header.message_type() {
                MessageType::Handshake => payload::<Handshake>(h, &mut bytes)?,
                MessageType::ConfirmReq => payload::<ConfirmReq>(h, &mut bytes)?,
                MessageType::ConfirmAck => payload::<ConfirmAck>(h, &mut bytes)?,
                MessageType::Keepalive => payload::<Keepalive>(h, &mut bytes)?,
                MessageType::TelemetryReq => payload::<Empty>(h, &mut bytes)?,
                MessageType::Publish => payload::<Publish>(h, &mut bytes)?,
                _ => todo!("{:?}", header),
            };
            println!(
                "{} {}",
                direction_text,
                color.paint(header.to_short_string())
            );
            let msg = format!("{:#?}", p.as_ref());
            println!("{}", color.paint(msg));
        }

        direction.swap();
    }

    Ok(())
}

pub fn payload<T: 'static + Wire>(
    header: Option<&Header>,
    bytes: &mut Bytes,
) -> anyhow::Result<Box<dyn Wire>> {
    let len = T::len(header)?;
    let data = bytes.slice(len)?;
    let payload: T = T::deserialize(header, data)?;
    Ok(Box::new(payload))
}
