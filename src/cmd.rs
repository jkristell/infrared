/// Remote control command trait
pub trait Command {
    /// Construct a command
    fn construct(addr: u32, data: u32) -> Option<Self>
    where
        Self: core::marker::Sized;

    /// Command address
    fn address(&self) -> u32;

    /// Get the data associated with the command
    fn data(&self) -> u32;

    /// Protocol
    fn protocol(&self) -> Protocol;

    /// Command as pulses
    fn pulses(&self, buf: &mut [u16]) -> usize;

    /// Pulses scaled by shifting
    fn pulses_shift_scaled(&self, buf: &mut [u16], shift: usize) -> usize {
        let len = self.pulses(buf);
        buf.iter_mut().take(len).for_each(|pulse| *pulse >>= shift);
        len
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
#[non_exhaustive]
/// Protocol
pub enum Protocol {
    Nec = 1,
    Nec16 = 2,
    NecSamsung = 3,
    Rc5 = 5,
    Rc6 = 6,
    Sbp = 7,
    Unknown = 255,
}

impl From<u8> for Protocol {
    fn from(u: u8) -> Self {
        match u {
            1 => Protocol::Nec,
            2 => Protocol::Nec16,
            3 => Protocol::NecSamsung,
            5 => Protocol::Rc5,
            6 => Protocol::Rc6,
            7 => Protocol::Sbp,
            _ => Protocol::Unknown,
        }
    }
}
