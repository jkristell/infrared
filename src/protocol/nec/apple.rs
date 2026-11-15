//! NEC variant used by Apple remotes

use crate::protocol::{
    Command, ProtocolId,
    nec::{NEC_STANDARD_TIMING, NecCommandVariant, NecPulseLen},
};

const APPLE_VENDOR_ID: u32 = 0x43f;

/// Apple NEC command
///
/// As a [`Command`] the address is the `device_id` and the command is
/// `command_page << 7 | command`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AppleNecCommand {
    /// Command page, 5 bits
    pub command_page: u8,
    /// Command, 7 bits
    pub command: u8,
    /// Device id, changeable by pairing the remote
    pub device_id: u8,
    /// Repeat
    pub repeat: bool,
}

impl NecCommandVariant for AppleNecCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        let vendor = (bits >> 5) & 0x7FF;

        // Odd parity
        vendor == APPLE_VENDOR_ID && (bits.count_ones() & 0x1) == 1
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        if !Self::validate(bits) {
            return None;
        }

        // Bit 0..5: command page, 5..16: vendor, 16: parity,
        // 17..24: command, 24..32: device id
        Some(AppleNecCommand {
            command_page: (bits & 0x1F) as u8,
            command: ((bits >> 17) & 0x7F) as u8,
            device_id: ((bits >> 24) & 0xFF) as u8,
            repeat,
        })
    }

    fn pack(&self) -> u32 {
        let bits = u32::from(self.command_page & 0x1F)
            | (APPLE_VENDOR_ID << 5)
            | (u32::from(self.command & 0x7F) << 17)
            | (u32::from(self.device_id) << 24);

        // Set the parity bit to make the parity odd
        let parity = !bits.count_ones() & 0x1;
        bits | (parity << 16)
    }

    fn repeat(&self) -> bool {
        self.repeat
    }
}

impl Command for AppleNecCommand {
    const PROTOCOL: ProtocolId = ProtocolId::AppleNec;

    fn address(&self) -> u32 {
        self.device_id.into()
    }

    fn command(&self) -> u32 {
        (u32::from(self.command_page) << 7) | u32::from(self.command)
    }

    fn is_repeat(&self) -> bool {
        self.repeat
    }

    fn raw(&self) -> u64 {
        self.pack().into()
    }

    fn from_parts(address: u32, command: u32) -> Option<Self> {
        if command >= 1 << 12 {
            return None;
        }
        Some(AppleNecCommand {
            command_page: (command >> 7) as u8,
            command: (command & 0x7F) as u8,
            device_id: address.try_into().ok()?,
            repeat: false,
        })
    }
}
