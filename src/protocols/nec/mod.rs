#[macro_use]
pub mod receiver;
pub mod transmitter;

#[cfg(test)]
mod tests;

pub use receiver::{NecError, NecTypeReceiver, NecResult};
pub use transmitter::NecTypeTransmitter;
use crate::ProtocolId;

pub struct StandardType;
pub struct SamsungType;

pub type NecReceiver = NecTypeReceiver<StandardType>;
pub type NecSamsungReceiver = NecTypeReceiver<SamsungType>;

pub type NecTransmitter = NecTypeTransmitter<StandardType>;
pub type NecSamsungTransmitter = NecTypeTransmitter<SamsungType>;

#[derive(Debug, Copy, Clone)]
/// A Nec Command
pub struct NecCommand {
    pub addr: u8,
    pub cmd: u8,
}

impl NecCommand {

    pub fn new(addr: u8, cmd: u8) -> Self {
        NecCommand {
            addr,
            cmd
        }
    }

    pub fn from_bits(bits: u32) -> Self {
        let addr = ((bits) & 0xFF) as u8;
        let cmd = ((bits >> 16) & 0xFF) as u8;
        Self {addr, cmd}
    }
}

pub trait NecTypeTrait {
    const PULSEDISTANCE: Pulsedistance;
    const PROTOCOL: ProtocolId;

    fn encode_command(cmd: NecCommand) -> u32;
}

impl NecTypeTrait for StandardType {
    const PROTOCOL: ProtocolId = ProtocolId::Nec;
    const PULSEDISTANCE: Pulsedistance = Pulsedistance {
        header_high: 9000,
        header_low: 4500,
        repeat_low: 2250,
        data_high: 560,
        zero_low: 560,
        one_low: 1690,
    };

    fn encode_command(NecCommand {addr, cmd}: NecCommand) -> u32 {
        let addr = u32::from(addr) | u32::from(!addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
    }
}

impl NecTypeTrait for SamsungType {
    const PROTOCOL: ProtocolId = ProtocolId::NecSamsung;
    const PULSEDISTANCE: Pulsedistance = Pulsedistance {
        header_high: 4500,
        header_low: 4500,
        repeat_low: 2250,
        zero_low: 560,
        data_high: 560,
        one_low: 1690,
    };

    fn encode_command(NecCommand {addr, cmd}: NecCommand) -> u32 {
        // Address is inverted and command is repeated
        let addr = u32::from(addr) | u32::from(addr) << 8;
        let cmd = u32::from(cmd) << 16 | u32::from(!cmd) << 24;
        addr | cmd
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
