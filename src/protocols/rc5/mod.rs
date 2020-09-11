//! Rc5
use crate::Command;

use core::convert::TryInto;
pub use receiver::Rc5;
pub use send::Rc5Sender;
use crate::cmd::Protocol;

pub mod receiver;
mod send;
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
    pub toggle: u8,
}

impl Rc5Command {
    pub const fn new(addr: u8, cmd: u8, toggle: bool) -> Self {
        Self {
            addr,
            cmd,
            start: 0b11,
            toggle: toggle as u8,
        }
    }

    pub const fn from_bits(bits: u16) -> Self {
        let addr = ((bits & ADDR_MASK) >> ADDR_SHIFT) as u8;
        let cmd = (bits & CMD_MASK) as u8;
        let start = ((bits & START_MASK) >> START_SHIFT) as u8;
        let toggle = ((bits & TOGGLE_MASK) >> TOGGLE_SHIFT) as u8;

        Self {
            addr,
            cmd,
            start,
            toggle,
        }
    }

    pub fn to_bits(&self) -> u16 {
        u16::from(self.addr) << ADDR_SHIFT
            | u16::from(self.cmd)
            | u16::from(self.toggle) << TOGGLE_SHIFT
            | u16::from(self.start) << START_SHIFT
    }
}

impl Command for Rc5Command {
    fn construct(addr: u32, cmd: u32) -> Option<Rc5Command> {
        let addr: u8 = addr.try_into().ok()?;
        let cmd = cmd.try_into().ok()?;

        Some(Rc5Command::new(addr, cmd, false))
    }

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn data(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> Protocol {
        Protocol::Rc5
    }
}

