//! Infrared send functionality

mod sender;
pub use sender::*;
mod buffer;
pub use buffer::*;

pub trait InfraredSender {
    type Cmd;
    fn with_samplerate(samplerate: u32) -> Self;

    fn cmd_pulsedata(&self, cmd: Self::Cmd, buf: &mut [u16]) -> usize;
}


pub trait ToPulsedata {
    fn to_pulsedata(&self, buf: &mut [u16]) -> usize;
}
