//! Philips Rc5

pub mod decoder;
use crate::protocol::{Command, Protocol, ProtocolId};

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

/// RC5 command, 5 bit address and 6 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rc5Command {
    /// Address
    pub address: u8,
    /// Command
    pub command: u8,
    /// The two start bits. The second start bit is the inverted bit 6 of the
    /// command in the RC5X extension
    pub start: u8,
    /// Toggle bit, flips on every new key press
    pub toggle: bool,
}

impl Rc5Command {
    /// Create a new command
    pub const fn new(address: u8, command: u8, toggle: bool) -> Self {
        Self {
            address,
            command,
            start: 0b11,
            toggle,
        }
    }

    /// Unpack the 14 bits of a frame
    pub const fn unpack(bits: u16) -> Self {
        let address = ((bits & ADDR_MASK) >> ADDR_SHIFT) as u8;
        let command = (bits & CMD_MASK) as u8;
        let start = ((bits & START_MASK) >> START_SHIFT) as u8;
        let toggle = (bits & TOGGLE_MASK) != 0;

        Self {
            address,
            command,
            start,
            toggle,
        }
    }

    /// Pack the command into the 14 bits of a frame
    pub fn pack(&self) -> u16 {
        (u16::from(self.address) << ADDR_SHIFT)
            | u16::from(self.command)
            | (u16::from(self.toggle) << TOGGLE_SHIFT)
            | (u16::from(self.start) << START_SHIFT)
    }
}

impl Command for Rc5Command {
    const PROTOCOL: ProtocolId = ProtocolId::Rc5;

    fn address(&self) -> u32 {
        self.address.into()
    }

    fn command(&self) -> u32 {
        self.command.into()
    }

    fn is_repeat(&self) -> bool {
        self.toggle
    }

    fn raw(&self) -> u64 {
        self.pack().into()
    }

    fn from_parts(address: u32, command: u32) -> Option<Rc5Command> {
        if address >= 1 << 5 || command >= 1 << 6 {
            return None;
        }
        Some(Rc5Command::new(address as u8, command as u8, false))
    }
}
