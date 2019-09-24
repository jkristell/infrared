#[macro_use]
pub mod remotes;
pub mod receiver;
pub mod transmitter;

pub use receiver::{NecCommand, NecError, NecTypeReceiver, NecResult};
pub use transmitter::NecTypeTransmitter;

pub type NecReceiver = NecTypeReceiver<StandardType>;
pub type NecSamsungReceiver = NecTypeReceiver<SamsungType>;

pub type NecTransmitter = NecTypeTransmitter<SamsungType>;

pub struct SamsungType;
pub struct StandardType;

pub trait NecTypeTrait {
    const TIMING: Timing;
    const ADDR_BITS: usize;
    const CMD_BITS: usize;
}

impl NecTypeTrait for SamsungType {
    const TIMING: Timing = Timing {
        header_high: 4500,
        header_low: 4500,
        repeat_low: 2250,
        zero_low: 560,
        data_high: 560,
        one_low: 1690,
    };
    const ADDR_BITS: usize = 8;
    const CMD_BITS: usize = 8;
}

impl NecTypeTrait for StandardType {
    const TIMING: Timing = Timing {
        header_high: 9000,
        header_low: 4500,
        repeat_low: 2250,
        data_high: 560,
        zero_low: 560,
        one_low: 1690,
    };
    const ADDR_BITS: usize = 8;
    const CMD_BITS: usize = 8;
}


pub struct Timing {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

#[cfg(test)]
mod tests {
    use crate::protocols::nec::NecReceiver;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);

        let _recv = NecReceiver::new(40_000);




    }
}
