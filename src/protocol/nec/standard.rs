//! The Standard NEC variant

use crate::{
    cmd::{AddressCommand, Command},
    protocol::nec::{NecCommandVariant, NecPulseLen, NEC_STANDARD_TIMING},
    ProtocolId,
};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NecCommand {
    pub addr: u8,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandVariant for NecCommand {
    const PULSE_DISTANCE: &'static NecPulseLen = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0xFF
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let addr = ((bits) & 0xFF) as u8;
        let cmd = ((bits >> 16) & 0xFF) as u8;

        Some(NecCommand { addr, cmd, repeat })
    }

    fn pack(&self) -> u32 {
        let addr = u32::from(self.addr) | (u32::from(!self.addr) & 0xFF) << 8;
        let cmd = u32::from(self.cmd) << 16 | u32::from(!self.cmd) << 24;
        addr | cmd
    }
}

impl Command for NecCommand {
    fn is_repeat(&self) -> bool {
        self.repeat
    }
}

impl AddressCommand for NecCommand {
    const ID: ProtocolId = ProtocolId::Nec;

    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(NecCommand {
            addr: addr as u8,
            cmd: cmd as u8,
            repeat: false,
        })
    }
}
