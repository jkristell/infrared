#[macro_use]
pub mod remotes;
pub mod receiver;
pub mod transmitter;

pub use receiver::{NecReceiver, NecCommand, NecResult, NecError};
pub use transmitter::{NecTransmitter};

pub enum NecType {
    Standard,
    Samsung,
}

pub struct Timing {
    header_htime: u32,
    header_ltime: u32,
    repeat_ltime: u32,

    data_htime: u32,
    zero_ltime: u32,
    one_ltime: u32,
}


const STANDARD_TIMING: Timing = Timing {
    header_htime: 9000,
    header_ltime: 4500,
    repeat_ltime: 2250,
    data_htime: 560,
    zero_ltime: 560,
    one_ltime: 1690,
};

const SAMSUNG_TIMING: Timing = Timing {
    header_htime: 4500,
    header_ltime: 4500,
    repeat_ltime: 2250,
    zero_ltime: 560,
    data_htime: 560,
    one_ltime: 1690,
};


