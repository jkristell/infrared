//! The Standard NEC variant

use crate::protocol::{
    Command, ProtocolId,
    nec::{NEC_STANDARD_TIMING, NecCommandVariant, NecPulseLen},
};

/// NEC command, 8 bit address and 8 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NecCommand {
    /// Address
    pub address: u8,
    /// Command
    pub command: u8,
    /// Repeat
    pub repeat: bool,
}

impl NecCommandVariant for NecCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0xFF
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let address = ((bits) & 0xFF) as u8;
        let command = ((bits >> 16) & 0xFF) as u8;

        Some(NecCommand {
            address,
            command,
            repeat,
        })
    }

    fn pack(&self) -> u32 {
        let address = u32::from(self.address) | ((u32::from(!self.address) & 0xFF) << 8);
        let command = (u32::from(self.command) << 16) | (u32::from(!self.command) << 24);
        address | command
    }

    fn repeat(&self) -> bool {
        self.repeat
    }
}

impl Command for NecCommand {
    const PROTOCOL: ProtocolId = ProtocolId::Nec;

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
        Some(NecCommand {
            address: address.try_into().ok()?,
            command: command.try_into().ok()?,
            repeat: false,
        })
    }
}
