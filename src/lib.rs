#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

mod protocols;
pub use protocols::*;

pub mod remotecontrol;
pub use remotecontrol::RemoteControl;

mod transmitter;
pub use transmitter::{TransmitterState, Transmitter};

#[cfg(feature="embedded-hal")]
pub use crate::transmitter::PwmTransmitter;

pub mod prelude {
    pub use crate::Receiver;
    pub use crate::Transmitter;
    pub use crate::ReceiverState;
    pub use crate::TransmitterState;
    #[cfg(feature="embedded-hal")]
    pub use crate::PwmTransmitter;
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum ReceiverState<CMD, ERR> {
    Idle,
    Receiving,
    Done(CMD),
    Error(ERR),
    Disabled,
}

/// Receiver trait
pub trait Receiver {
    /// The resulting command type
    type Cmd;
    /// Receive Error
    type Err;

    /// Sample
    fn sample(&mut self, pinval: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Sample on known edge
    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}


#[cfg(feature="protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

