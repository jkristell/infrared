#![no_std]

use core::convert::Into;

/// NEC protocol decoder
pub mod protocols;
/// Tracing protocol decoder
pub mod trace;

/// Remote controls
pub mod remote;

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
    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}


pub enum TransmitterState {
    Idle,
    Transmit(bool),
    Done,
    Err,
}

pub trait Transmitter {

    // Set command to be transmitted
    fn set_command<CMD: Into<u32>>(&mut self, cmd: CMD);

    // Step the transfer loop
    fn transmit(&mut self, ts: u32) -> TransmitterState;

    fn reset(&mut self) {}
}

