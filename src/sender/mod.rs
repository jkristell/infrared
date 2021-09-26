//! Infrared sender functionality

use crate::protocol::Protocol;

mod buffer;
#[cfg(feature = "embedded-hal")]
mod hal;
mod senders;

pub use buffer::*;
#[cfg(feature = "embedded-hal")]
pub use hal::*;
pub use senders::*;

pub trait ProtocolEncoder<const FREQ: usize>: Protocol {
    type EncoderData;
    const DATA: Self::EncoderData;

    fn encode(cmd: &Self::Cmd, buf: &mut [usize]) -> usize;
}
