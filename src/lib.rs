#![no_std]

use core::convert::Into;

/// NEC protocol decoder
pub use protocols::nec;
pub use protocols::philips;

pub mod trace;

/// Remote controls
pub mod remote;
pub use remote::RemoteControl;

mod protocols;

#[derive(PartialEq)]
/// Protocol decoder state
pub enum ReceiverState<CMD, ERR> {
    Idle,
    Receiving,
    Done(CMD),
    Err(ERR),
    Disabled,
}

/// Receiver trait
pub trait Receiver {
    /// The resulting command type
    type Command;
    /// Receive Error
    type ReceiveError;

    /// Register new event
    fn event(
        &mut self,
        rising: bool,
        timestamp: u32,
    ) -> ReceiverState<Self::Command, Self::ReceiveError>;
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
