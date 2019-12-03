//! Infrared

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
/// Remote Control Protocol Id
pub enum ProtocolId {
    /// Nec
    Nec = 1,
    /// Nec with 16 bit address
    Nec16 = 2,
    /// Nec - Samsung variant
    NecSamsung = 3,
    /// Philips Rc5
    Rc5 = 4,
    /// Philips Rc6
    Rc6 = 5,
    /// Samsung 36 bit protocol
    Sbp = 6,

    /// Logging
    Logging = 31,
}

/// Remote control command trait
pub trait Command {
    fn construct(addr: u16, cmd: u8) -> Self;
    /// Get the address from the command
    fn address(&self) -> u16;
    /// Get the command number
    fn command(&self) -> u8;
}

#[cfg(feature = "embedded-hal")]
mod hal;

mod protocols;
pub use protocols::*;

mod receiver;

pub use receiver::{
    ReceiverError,
    ReceiverState,
    ReceiverStateMachine,
};

#[cfg(feature = "embedded-hal")]
pub use hal::{
    InfraredReceiver,
    InfraredReceiver2,
    InfraredReceiver3,
    InfraredReceiver4,
    InfraredReceiver5,
};

#[cfg(feature = "protocol-debug")]
pub use receiver::ReceiverDebug;

mod transmitter;
pub use crate::transmitter::{Transmitter, TransmitterState};
#[cfg(feature = "embedded-hal")]
pub use transmitter::PwmTransmitter;

#[cfg(feature = "remotes")]
pub mod remotes;

