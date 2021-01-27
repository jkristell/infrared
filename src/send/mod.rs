//! Infrared send functionality

mod sender;
pub use sender::*;
mod buffer;
pub use buffer::*;


pub trait ToPulsedata {
    fn to_pulsedata(&self, buf: &mut [u16]) -> usize;
}
