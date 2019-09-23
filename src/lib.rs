#![no_std]

use core::convert::Into;

/// NEC protocol
pub use protocols::nec;
/// Rc5 Protocol
pub use protocols::rc5;
/// Rc6 Protocol
pub use protocols::rc6;

pub mod trace;

/// Remote controls
pub mod remote;
pub use remote::RemoteControl;

mod protocols;

#[derive(PartialEq, Copy, Clone)]
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
    fn sample(&mut self, pinval: bool, timestamp: u32) -> ReceiverState<Self::Cmd, Self::Err>;

    /// Register and edge
    fn edge(&mut self, rising: bool, sampledelta: u16) -> ReceiverState<Self::Cmd, Self::Err>;

    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}

pub enum TransmitterState {
    /// Transmitter is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error state
    Err,
}

pub trait Transmitter {
    /// Initialize transfer
    fn init<CMD: Into<u32>>(&mut self, cmd: CMD);

    /// Step the transfer loop
    fn step(&mut self, ts: u32) -> TransmitterState;

    /// Reset the transmitter
    fn reset(&mut self);
}
