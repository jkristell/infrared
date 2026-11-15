//! Infrared sender functionality

use crate::protocol::Protocol;

mod buffer;
#[cfg(feature = "embedded-hal")]
mod hal;
mod senders;

pub(crate) use buffer::PulseBuffer;
#[cfg(feature = "embedded-hal")]
pub use hal::PwmSender;
pub use senders::*;

pub trait ProtocolEncoder<const FREQ: u32>: Protocol {
    type EncoderData;
    const DATA: Self::EncoderData;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize;
}
