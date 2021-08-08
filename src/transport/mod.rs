use crate::encoding::to_hex;
use crate::node::{Header, Wire};
use std::fmt::Debug;
use tracing::{instrument, trace};

pub mod header;
pub mod wire;

#[macro_export]
macro_rules! handle {
    ($self: ident, $fun:ident, $header:ident, $mut_incoming_buffer: ident) => {{
        let sh = Some(&$header);
        let payload = crate::transport::recv(sh, $mut_incoming_buffer)
            .with_context(|| format!("Receiving payload for {:?}", $header))?;

        if let Some(payload) = payload {
            // TODO: reinstate
            // match &self.last_annotation {
            //     Some(a) => info!("{} {:?}", a, &payload),
            //     None => debug!("{:?}", &payload),
            // };

            $self
                .$fun(&$header, payload)
                .await
                .with_context(|| format!("Handling payload for {:?}", $header))?;
        } else {
        }
    };};
}

/// Receive from the incoming buffer for type `T`. Will return None if there aren't enough
/// bytes available.
#[instrument(skip(header, incoming_buffer))]
pub fn recv<T: Wire + Debug>(
    header: Option<&Header>,
    incoming_buffer: &mut Vec<u8>,
) -> anyhow::Result<Option<T>> {
    let bytes = T::len(header)?;
    if incoming_buffer.len() < bytes {
        trace!(
            "Not enough bytes. Got {}, expected {}.",
            incoming_buffer.len(),
            bytes
        );
        return Ok(None);
    }

    let buffer = incoming_buffer.drain(..bytes).collect::<Vec<u8>>();
    trace!("HEX: {}", to_hex(&buffer));
    let result = T::deserialize(header, &buffer)?;
    Ok(Some(result))
}
