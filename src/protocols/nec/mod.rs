pub mod remotes;
pub mod receiver;
pub mod transmitter;

// NEC Header
//
// _/'''''''''\_____ DATA
//  |--- 9 ---| 4.5 |

// Samsung TV Header
//
//_/'''''\_____
// | 4.5 | 4.5 |

pub enum NecType {
    Nec,
    Samsung,
}

pub struct Timing {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    zero_low: u32,
    zero_high: u32,
    zero: u32,
    one_low: u32,
    one_high: u32,
    one: u32,
}


const GENERIC_TIMING: Timing = Timing {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    zero: 1250,
    zero_low: 560,
    zero_high: 560,
    one: 2250,
    one_high: 560,
    one_low: 1690,
};

const SAMSUNG_TIMING: Timing = Timing {
    header_high: 4500,
    header_low: 4500,
    repeat_low: 2250,
    zero: 1250,
    zero_low: 560,
    zero_high: 560,
    one: 2250,
    one_high: 560,
    one_low: 1690,
};



