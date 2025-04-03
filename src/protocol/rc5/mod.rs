//! Philips Rc5

use core::convert::TryInto;

pub mod decoder;
use crate::{
    cmd::{AddressCommand, Command},
    protocol::Protocol,
    ProtocolId,
};

pub mod encoder;

#[cfg(test)]
mod tests;

const ADDR_MASK: u16 = 0b_0000_0111_1100_0000;
const CMD_MASK: u16 = 0b_0000_0000_0011_1111;
const START_MASK: u16 = 0b_0011_0000_0000_0000;
const TOGGLE_MASK: u16 = 0b_0000_1000_0000_0000;

const ADDR_SHIFT: u32 = 6;
const START_SHIFT: u32 = 12;
const TOGGLE_SHIFT: u32 = 11;

/// Philips Rc5
pub struct Rc5;

impl Protocol for Rc5 {
    type Cmd = Rc5Command;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
        (u16::from(self.addr) << ADDR_SHIFT)
            | u16::from(self.cmd)
            | (u16::from(self.toggle) << TOGGLE_SHIFT)
            | (u16::from(self.start) << START_SHIFT)
    }
}

impl Command for Rc5Command {
    fn is_repeat(&self) -> bool {
        self.toggle
    }
}

impl AddressCommand for Rc5Command {
    const ID: ProtocolId = ProtocolId::Rc5;

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn create(addr: u32, cmd: u32) -> Option<Rc5Command> {
        let addr: u8 = addr.try_into().ok()?;
        let cmd = cmd.try_into().ok()?;

        Some(Rc5Command::new(addr, cmd, false))
    }
}
