//! Infrared send functionality

mod sender;
pub use sender::*;
mod buffer;
pub use buffer::*;

pub trait InfraredSender {
    fn with_samplerate(samplerate: u32) -> Self;
}


pub trait ToPulsedata {
    fn to_pulsedata(&self, buf: &mut [u16]) -> usize;
}
