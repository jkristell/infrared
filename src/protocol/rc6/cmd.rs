use crate::protocol::{Command, ProtocolId};

/// RC6 (mode 0) command, 8 bit address and 8 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rc6Command {
    /// Address
    pub address: u8,
    /// Command
    pub command: u8,
    /// Toggle bit, flips on every new key press
    pub toggle: bool,
}

impl Rc6Command {
    /// Create a new command
    pub const fn new(address: u8, command: u8) -> Self {
        Self {
            address,
            command,
            toggle: false,
        }
    }

    /// Create a command from the 16 data bits of a frame
    pub const fn from_bits(bits: u16, toggle: bool) -> Self {
        let address = (bits >> 8) as u8;
        let command = (bits & 0xFF) as u8;
        Self {
            address,
            command,
            toggle,
        }
    }
}

impl Command for Rc6Command {
    const PROTOCOL: ProtocolId = ProtocolId::Rc6;

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
        (u64::from(self.address) << 8) | u64::from(self.command)
    }

    fn from_parts(address: u32, command: u32) -> Option<Self> {
        Some(Rc6Command::new(
            address.try_into().ok()?,
            command.try_into().ok()?,
        ))
    }
}
