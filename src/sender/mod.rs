//! Infrared sender functionality

use crate::protocol::Protocol;

mod buffer;
mod senders;

pub(crate) use buffer::PulseBuffer;
pub use senders::*;

pub trait ProtocolEncoder<const FREQ: u32>: Protocol {
    type EncoderData;
    const DATA: Self::EncoderData;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize;
}
