use crate::protocols::nec::{NecPulseDistance, NecCommandTrait, NEC_SAMSUNG_TIMING, NEC_STANDARD_TIMING};
#[cfg(feature = "remotes")]
use crate::remotecontrol::AsButton;
use crate::ProtocolId;

/*
 * -------------------------------------------------------------------------
 *  The Standard NEC variant
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecCommand {
    pub addr: u8,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandTrait for NecCommand {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

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

#[cfg(feature = "remotes")]
impl AsButton for NecCommand {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> crate::ProtocolId {
        crate::ProtocolId::Nec
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(NecCommand {
            addr: addr as u8,
            cmd: cmd as u8,
            repeat: false,
        })
    }
}


/*
 * -------------------------------------------------------------------------
 *  NEC variant with 16 bit address
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command with 16 bit address
pub struct Nec16Command {
    pub addr: u16,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandTrait for Nec16Command {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let addr = (bits & 0xFFFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;

        Some(Nec16Command { addr, cmd, repeat })
    }

    fn pack(&self) -> u32 {
        let addr = u32::from(self.addr);
        let cmd = u32::from(self.cmd) << 16 | u32::from(!self.cmd) << 24;
        addr | cmd
    }
}

/*
 * -------------------------------------------------------------------------
 *  NEC Samsung Command variant
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecSamsungCommand {
    pub addr: u8,
    pub cmd: u8,
    pub repeat: bool,
}

impl NecCommandTrait for NecSamsungCommand {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_SAMSUNG_TIMING;

    fn validate(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0x00
    }

    fn unpack(bits: u32, repeat: bool) -> Option<Self> {
        let addr = (bits & 0xFF) as u8;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        Some(NecSamsungCommand { addr, cmd, repeat })
    }

    fn pack(&self) -> u32 {
        let addr = u32::from(self.addr) | u32::from(self.addr) << 8;
        let cmd = u32::from(self.cmd) << 16 | u32::from(!self.cmd) << 24;
        addr | cmd
    }
}

#[cfg(feature = "remotes")]
impl AsButton for NecSamsungCommand {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn protocol(&self) -> ProtocolId {
        ProtocolId::NecSamsung
    }

    fn create(addr: u32, cmd: u32) -> Option<Self> {
        Some(NecSamsungCommand {
            addr: addr as u8,
            cmd: cmd as u8,
            repeat: false,
        })
    }
}

/*
 * -------------------------------------------------------------------------
 *  Apple Nec variant
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NecAppleCommand {
    pub command_page: u8,
    pub command: u8,
    pub device_id: u8,
    pub repeat: bool,
}

impl NecCommandTrait for NecAppleCommand {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

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

        Some(NecAppleCommand {
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

#[cfg(feature = "remotes")]
impl AsButton for NecAppleCommand {
    fn address(&self) -> u32 {
        0
    }

    fn command(&self) -> u32 {
        u32::from(self.command_page << 7 | self.command)
    }

    fn protocol(&self) -> ProtocolId {
        ProtocolId::NecApple
    }

    fn create(_addr: u32, _cmd: u32) -> Option<Self> {
        None
    }
}

/*
 * -------------------------------------------------------------------------
 *  Nec Raw variant. Useful for debugging
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command without parsing of bit meaning
pub struct NecRawCommand {
    pub bits: u32,
}

impl NecCommandTrait for NecRawCommand {
    const PULSE_DISTANCE: &'static NecPulseDistance = NEC_STANDARD_TIMING;

    fn validate(_bits: u32) -> bool {
        true
    }

    fn unpack(bits: u32, _repeat: bool) -> Option<Self> {
        Some(NecRawCommand { bits })
    }

    fn pack(&self) -> u32 {
        self.bits
    }
}
