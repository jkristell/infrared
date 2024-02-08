//! Infrared sender functionality

use crate::protocol::Protocol;

mod buffer;
#[cfg(feature = "embedded-hal-02")]
mod hal;
mod senders;

pub use buffer::*;
#[cfg(feature = "embedded-hal-02")]
pub use hal::*;
pub use senders::*;

pub trait ProtocolEncoder<const FREQ: u32>: Protocol {
    type EncoderData;
    const DATA: Self::EncoderData;

    fn encode(cmd: &Self::Cmd, buf: &mut [u32]) -> usize;
}
