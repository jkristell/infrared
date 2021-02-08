//! Infrared send functionality

mod sender;
pub use sender::*;
mod buffer;
pub use buffer::*;
use crate::protocolid::InfraredProtocol;

pub trait InfraredSenderState {
    fn create(samplerate: u32) -> Self;
}

pub trait InfraredSender: InfraredProtocol {
    type State: InfraredSenderState;

    fn with_samplerate(samplerate: u32) -> Self;

    fn sender_state(samplerate: u32) -> Self::State {
        Self::State::create(samplerate)
    }

    fn cmd_pulsedata(
                     state: &Self::State,
                     cmd: &Self::Cmd, buf: &mut [u16]) -> usize;
}


pub trait ToPulsedata {
    fn to_pulsedata(&self, buf: &mut [u16]) -> usize;
}
