//! Samsung32, the NEC variant used by Samsung

use crate::protocol::{
    Command, ProtocolId,
    nec::{NEC_SAMSUNG_TIMING, NecCommandVariant, NecPulseLen},
};

/// Samsung32 command, 8 bit address and 8 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Samsung32Command {
    /// Address
    pub address: u8,
    /// Command
    pub command: u8,
    /// Repeat
    pub repeat: bool,
}

impl NecCommandVariant for Samsung32Command {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_SAMSUNG_TIMING;
    const SHORT_REPEAT: bool = false;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0x00
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let address = (bits & 0xFF) as u8;
        let command = ((bits >> 16) & 0xFF) as u8;
        Some(Samsung32Command {
            address,
            command,
            repeat,
        })
    }

    fn pack(&self) -> u32 {
        let address = u32::from(self.address) | (u32::from(self.address) << 8);
        let command = (u32::from(self.command) << 16) | (u32::from(!self.command) << 24);
        address | command
    }

    fn repeat(&self) -> bool {
        self.repeat
    }
}

impl Command for Samsung32Command {
    const PROTOCOL: ProtocolId = ProtocolId::Samsung32;

    fn address(&self) -> u32 {
        self.address.into()
    }

    fn command(&self) -> u32 {
        self.command.into()
    }

    fn is_repeat(&self) -> bool {
        self.repeat
    }

    fn raw(&self) -> u64 {
        self.pack().into()
    }

    fn from_parts(address: u32, command: u32) -> Option<Self> {
        Some(Samsung32Command {
            address: address.try_into().ok()?,
            command: command.try_into().ok()?,
            repeat: false,
        })
    }
}
