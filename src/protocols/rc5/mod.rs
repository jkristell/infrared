//! Philips Rc5

use crate::{
    send::ToPulsedata,
    ProtocolId,
};
use core::convert::TryInto;

#[cfg(feature = "remotes")]
use crate::remotecontrol::AsButton;

pub mod receiver;
pub use receiver::Rc5;
#[cfg(test)]
mod tests;

const ADDR_MASK: u16 = 0b_0000_0111_1100_0000;
const CMD_MASK: u16 = 0b_0000_0000_0011_1111;
const START_MASK: u16 = 0b_0011_0000_0000_0000;
const TOGGLE_MASK: u16 = 0b_0000_1000_0000_0000;

const ADDR_SHIFT: u32 = 6;
const START_SHIFT: u32 = 12;
const TOGGLE_SHIFT: u32 = 11;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Rc5Command {
    pub addr: u8,
    pub cmd: u8,
    pub start: u8,
    pub toggle: bool,
}

impl Rc5Command {
    pub const fn new(addr: u8, cmd: u8, toggle: bool) -> Self {
        Self {
            addr,
            cmd,
            start: 0b11,
            toggle,
        }
    }

    pub const fn unpack(bits: u16) -> Self {
        let addr = ((bits & ADDR_MASK) >> ADDR_SHIFT) as u8;
        let cmd = (bits & CMD_MASK) as u8;
        let start = ((bits & START_MASK) >> START_SHIFT) as u8;
        let toggle = (bits & TOGGLE_MASK) != 0;

        Self {
            addr,
            cmd,
            start,
            toggle,
        }
    }

    pub fn pack(&self) -> u16 {
        u16::from(self.addr) << ADDR_SHIFT
            | u16::from(self.cmd)
            | u16::from(self.toggle) << TOGGLE_SHIFT
            | u16::from(self.start) << START_SHIFT
    }
}

impl ToPulsedata for Rc5Command {
    fn to_pulsedata(&self, buf: &mut [u16]) -> usize {
        // Command as bits
        let bits = self.pack();

        // First bit is always one
        buf[0] = 0;
        let mut prev = true;
        let mut index = 1;

        for b in 0..13 {
            let cur = bits & (1 << (12 - b)) != 0;

            if prev == cur {
                buf[index] = 889;
                buf[index + 1] = 889;
                index += 2;
            } else {
                buf[index] = 889 * 2;
                index += 1;
            }

            prev = cur;
        }

        index
    }
}

#[cfg(feature = "remotes")]
impl AsButton for Rc5Command {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> ProtocolId {
        ProtocolId::Rc5
    }

    fn create(addr: u32, cmd: u32) -> Option<Rc5Command> {
        let addr: u8 = addr.try_into().ok()?;
        let cmd = cmd.try_into().ok()?;

        Some(Rc5Command::new(addr, cmd, false))
    }
}
