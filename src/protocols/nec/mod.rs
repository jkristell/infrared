#[macro_use]
pub mod remotes;
pub mod receiver;
pub mod transmitter;

pub use receiver::{NecCommand, NecError, NecReceiver, NecResult};
pub use transmitter::NecTransmitter;

pub enum NecType {
    Standard,
    Samsung,
}

pub struct Timing {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

const STANDARD_TIMING: Timing = Timing {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    data_high: 560,
    zero_low: 560,
    one_low: 1690,
};

const SAMSUNG_TIMING: Timing = Timing {
    header_high: 4500,
    header_low: 4500,
    repeat_low: 2250,
    zero_low: 560,
    data_high: 560,
    one_low: 1690,
};
