#[macro_use]
pub mod receiver;
pub mod transmitter;

#[cfg(test)]
mod tests;

use crate::ProtocolId;
pub use receiver::{NecError, NecResult, NecTypeReceiver};
pub use transmitter::NecTypeTransmitter;

pub struct StandardType;
pub struct SamsungType;
/// NecVariant with 16 bit adress, 8 bit command
pub struct Nec16Type;

pub type NecReceiver = NecTypeReceiver<StandardType>;
pub type NecSamsungReceiver = NecTypeReceiver<SamsungType>;

pub type NecTransmitter = NecTypeTransmitter<StandardType>;
pub type NecSamsungTransmitter = NecTypeTransmitter<SamsungType>;
pub type Nec16Receiver = NecTypeReceiver<Nec16Type>;

#[derive(Debug, Copy, Clone, PartialEq)]
/// A Nec Command
pub struct NecCommand {
    pub addr: u16,
    pub cmd: u8,
}

impl NecCommand {
    pub fn new(addr: u16, cmd: u8) -> Self {
        NecCommand { addr, cmd }
    }
}

pub trait NecTypeTrait {
    const PULSEDISTANCE: &'static Pulsedistance;
    const PROTOCOL: ProtocolId;

    fn encode_command(cmd: NecCommand) -> u32;
    fn decode_command(bits: u32) -> NecCommand;
    fn verify_command(bits: u32) -> bool;
}

impl NecTypeTrait for StandardType {
    const PROTOCOL: ProtocolId = ProtocolId::Nec;
    const PULSEDISTANCE: &'static Pulsedistance = &STANDARD_DIST;

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        let addr = u32::from(addr) | (u32::from(!addr) & 0xFF) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0xFF
    }
}

impl NecTypeTrait for Nec16Type {
    const PROTOCOL: ProtocolId = ProtocolId::Nec16;
    const PULSEDISTANCE: &'static Pulsedistance = &STANDARD_DIST;

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        let addr = u32::from(addr);
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFFFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF
    }
}

impl NecTypeTrait for SamsungType {
    const PROTOCOL: ProtocolId = ProtocolId::NecSamsung;
    const PULSEDISTANCE: &'static Pulsedistance = &Pulsedistance {
        header_high: 4500,
        header_low: 4500,
        repeat_low: 2250,
        zero_low: 560,
        data_high: 560,
        one_low: 1690,
    };

    fn encode_command(NecCommand { addr, cmd }: NecCommand) -> u32 {
        // Address is inverted and command is repeated
        let addr = u32::from(addr) | u32::from(addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }

    fn decode_command(bits: u32) -> NecCommand {
        let addr = ((bits) & 0xFF) as u16;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        NecCommand { addr, cmd }
    }

    fn verify_command(bits: u32) -> bool {
        ((bits >> 24) ^ (bits >> 16)) & 0xFF == 0xFF && ((bits >> 8) ^ bits) & 0xFF == 0
    }
}

pub struct Pulsedistance {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

const STANDARD_DIST: Pulsedistance = Pulsedistance {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    data_high: 560,
    zero_low: 560,
    one_low: 1690,
};
