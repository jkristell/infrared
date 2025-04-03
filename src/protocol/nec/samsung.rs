//!NEC Samsung Command variant

use crate::{
    cmd::{AddressCommand, Command},
    protocol::nec::{NecCommandVariant, NecPulseLen, NEC_SAMSUNG_TIMING},
    ProtocolId,
};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SamsungNecCommand {
    pub addr: u8,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandVariant for SamsungNecCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_SAMSUNG_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0x00
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let addr = (bits & 0xFF) as u8;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        Some(SamsungNecCommand { addr, cmd, repeat })
    }

    fn pack(&self) -> u32 {
        let addr = u32::from(self.addr) | (u32::from(self.addr) << 8);
        let cmd = (u32::from(self.cmd) << 16) | (u32::from(!self.cmd) << 24);
        addr | cmd
    }
}

impl Command for SamsungNecCommand {
    fn is_repeat(&self) -> bool {
        self.repeat
    }
}

impl AddressCommand for SamsungNecCommand {
    const ID: ProtocolId = ProtocolId::NecSamsung;

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(SamsungNecCommand {
            addr: addr as u8,
            cmd: cmd as u8,
            repeat: false,
        })
    }
}
