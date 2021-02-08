//! Infrared send functionality

mod sender;
pub use sender::*;
mod buffer;
use crate::protocolid::InfraredProtocol;
pub use buffer::*;

pub trait InfraredSenderState {
    fn create(samplerate: u32) -> Self;
}

pub trait InfraredSender: InfraredProtocol {
    type State: InfraredSenderState;

    fn sender_state(samplerate: u32) -> Self::State {
        Self::State::create(samplerate)
    }

    fn cmd_pulsedata(state: &Self::State, cmd: &Self::Cmd, buf: &mut [u16]) -> usize;
}

