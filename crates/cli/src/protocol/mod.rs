//! SubTunnel control protocol — message types and wire format.
//!
//! Messages are serialized as JSON and framed with a 4-byte big-endian
//! length prefix. See [`codec`] for the framing layer and [`messages`]
//! for the message definitions.

pub mod codec;
pub mod messages;

pub use codec::{read_message, write_message};
pub use messages::ControlMessage;
