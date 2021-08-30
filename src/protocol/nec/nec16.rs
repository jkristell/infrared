//! NEC variant with 16 bit addresses and 8 bit data

use crate::protocol::nec::{NecCommandVariant, NecPulseDistance, NEC_STANDARD_TIMING};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Nec Command with 16 bit address
pub struct Nec16Command {
    pub addr: u16,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandVariant for Nec16Command {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let addr = (bits & 0xFFFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;

        Some(Nec16Command { addr, cmd, repeat })
    }

    fn pack(&self) -> u32 {
        let addr = u32::from(self.addr);
        let cmd = u32::from(self.cmd) << 16 | u32::from(!self.cmd) << 24;
        addr | cmd
    }
}
