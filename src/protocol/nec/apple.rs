//! Nec protocol variant from Apple

use crate::{cmd::{AddressCommand, Command}, protocol::nec::{NecCommandVariant, NecPulseLen, NEC_STANDARD_TIMING}, ProtocolId};
use crate::cmd::AnyCommand;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AppleNecCommand {
    pub command_page: u8,
    pub command: u8,
    pub device_id: u8,
    pub repeat: bool,
}

impl From<AppleNecCommand> for AnyCommand {
    fn from(value: AppleNecCommand) -> Self {
        AnyCommand {
            protocol: ProtocolId::NecApple,
            address: value.address(),
            command: value.command(),
            repeat: value.is_repeat(),
        }
    }
}


impl NecCommandVariant for AppleNecCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        let vendor = ((bits >> 5) & 0x7FF) as u16;
        const APPLE_VENDOR_ID: u16 = 0x43f;

        vendor == APPLE_VENDOR_ID &&
            // Odd parity
            (bits.count_ones() & 0x1) == 1
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        if !Self::validate(bits) {
            return None;
        }
        // 5 Bits
        let command_page = (bits & 0x1F) as u8;
        // 11 Bits
        let _vendor = ((bits >> 5) & 0x7FF) as u16;

        // 1 Bit
        let _parity_bit = (bits >> 16) & 0x1;
        // 7 Bits
        let command = ((bits >> 17) & 0x7F) as u8;
        // 8 Bits (Changable by pairing)
        let device_id = ((bits >> 24) & 0xFF) as u8;

        Some(AppleNecCommand {
            command_page,
            command,
            device_id,
            repeat,
        })
    }

    fn pack(&self) -> u32 {
        unimplemented!()
    }
}

impl Command for AppleNecCommand {
    fn is_repeat(&self) -> bool {
        self.repeat
    }
}

impl AddressCommand for AppleNecCommand {
    fn address(&self) -> u32 {
        0
    }

    fn command(&self) -> u32 {
        u32::from(self.command_page << 7 | self.command)
    }

    fn create(_addr: u32, _cmd: u32) -> Option<Self> {
        None
    }
}
