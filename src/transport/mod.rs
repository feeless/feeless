use crate::encoding::to_hex;
use crate::node::{Header, Wire};
use crate::transport::RecvResult::{NotEnoughBytes, Received};
use anyhow::Context;
use std::collections::VecDeque;
use std::fmt::Debug;
use tracing::{instrument, trace};

pub mod header;
pub mod wire;

pub enum RecvResult<T: Wire + Debug> {
    Received(T),
    NotEnoughBytes,
}

#[macro_export]
macro_rules! handle {
    ($self: expr, $fun:ident, $header:ident, $mut_incoming_buffer: ident) => {{
        let sh = Some(&$header);
        let received = crate::transport::recv(sh, $mut_incoming_buffer)
            .with_context(|| format!("Receiving payload for {:?}", $header))?;

        match received {
            Received(payload) => {
                // TODO: reinstate
                // match &self.last_annotation {
                //     Some(a) => info!("{} {:?}", a, &payload),
                //     None => debug!("{:?}", &payload),
                // };

                $self
                    .$fun(&$header, payload)
                    .await
                    .with_context(|| format!("Handling payload for {:?}", $header))?;
                ParserState::ReceivingHeader
            }
            RecvResult::NotEnoughBytes => ParserState::ReceivingPayload($header),
        }
    };};
}

/// Receive from the incoming buffer for type `T`. Will return None if there aren't enough
/// bytes available.
#[instrument(skip(header, incoming_buffer))]
pub fn recv<T: Wire + Debug>(
    header: Option<&Header>,
    incoming_buffer: &mut VecDeque<u8>,
) -> anyhow::Result<RecvResult<T>> {
    let bytes = T::len(header)?;
    if incoming_buffer.len() < bytes {
        trace!(
            "Not enough bytes. Got {}, expected {}.",
            incoming_buffer.len(),
            bytes
        );
        return Ok(NotEnoughBytes);
    }

    let buffer = incoming_buffer.drain(..bytes).collect::<Vec<u8>>();
    trace!("HEX: {}", to_hex(&buffer));
    let result = T::deserialize(header, &buffer)?;
    Ok(Received(result))
}

/// Receive from the incoming buffer for a `Header`. Will return None if there aren't enough
/// bytes available.
#[instrument(skip(incoming_buffer))]
pub fn recv_header(incoming_buffer: &mut VecDeque<u8>) -> anyhow::Result<RecvResult<Header>> {
    recv(None, incoming_buffer)
}

/// Receive from the incoming buffer for type `T` given a header. Will return None if there aren't enough
/// bytes available.
#[instrument(skip(header, incoming_buffer))]
pub fn recv_payload<T: Wire + Debug>(
    header: &Header,
    incoming_buffer: &mut VecDeque<u8>,
) -> anyhow::Result<RecvResult<T>> {
    recv(Some(header), incoming_buffer)
}

// pub fn parse_message<T: Wire + Debug>(
//     header: &Header,
//     incoming_buffer: &mut VecDeque<u8>,
// ) -> anyhow::Result<&ParserState> {
//     let message_type = header.message_type();
//     let bau = recv_payload(header, incoming_buffer)?;
//     match bau {
//         RecvResult<>
//     }
// }
