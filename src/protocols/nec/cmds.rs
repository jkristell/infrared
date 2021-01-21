use crate::protocols::nec::{NecCommandTrait, NecTiming, SamsungTiming, StandardTiming};
use crate::remotecontrol::AsRemoteControlButton;
use crate::PulseLengths;

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

impl AsRemoteControlButton for NecCommand {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn make(addr: u32, cmd: u32) -> Option<Self> {
        Some(NecCommand {
            addr: addr as u8,
            cmd: cmd as u8,
            repeat: false,
        })
    }
}

impl NecCommandTrait<StandardTiming> for NecCommand {
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

impl PulseLengths for NecCommand {
    fn encode(&self, b: &mut [u16]) -> usize {
        self.to_pulselengths(b)
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

impl NecCommandTrait<StandardTiming> for Nec16Command {
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

impl PulseLengths for Nec16Command {
    fn encode(&self, b: &mut [u16]) -> usize {
        self.to_pulselengths(b)
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

impl NecCommandTrait<SamsungTiming> for NecSamsungCommand {
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

impl AsRemoteControlButton for NecSamsungCommand {
    fn address(&self) -> u32 {
        self.addr.into()
    }

    fn command(&self) -> u32 {
        self.cmd.into()
    }

    fn make(addr: u32, cmd: u32) -> Option<Self> {
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

impl NecCommandTrait<StandardTiming> for NecAppleCommand {
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

impl AsRemoteControlButton for NecAppleCommand {
    fn address(&self) -> u32 {
        0
    }

    fn command(&self) -> u32 {
        u32::from(self.command_page << 7 | self.command)
    }

    fn make(_addr: u32, _cmd: u32) -> Option<Self> {
        None
    }
}

/*
 * -------------------------------------------------------------------------
 *  Nec Raw - variant useful for debugging
 * -------------------------------------------------------------------------
 */

#[derive(Debug, Copy, Clone, PartialEq)]
/// Nec Command
pub struct NecRawCommand {
    pub bits: u32,
}

impl<T: NecTiming> NecCommandTrait<T> for NecRawCommand {
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
