use core::convert::Into;

use crate::protocols::nec::{Timing, NecType, GENERIC_TIMING, SAMSUNG_TIMING};
use crate::{Transmitter, TransmitterState};

enum TransmitStateInternal {
    Idle,
    Start,
    HeaderHigh,
    HeaderLow,
    // Index, High/Low part
    DataLow(u32),
    DataHigh(u32),
    Done,
}


pub struct NecTransmitter {
    state: TransmitStateInternal,
    units: TimeUnits,
    last_ts: u32,
    cmd: u32,
}

struct TimeUnits {
    header_high: u32,
    header_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

impl TimeUnits {
    pub fn new(period: u32, timing: &Timing) -> Self {
        Self {
            header_high: timing.header_high / period,
            header_low: timing.header_low / period,
            zero_low: timing.zero_low / period,
            data_high: timing.data_high / period,
            one_low: timing.one_low / period,
        }
    }
}

impl NecTransmitter {
    pub fn new(nectype: NecType, period: u32) -> Self {

        let units = match nectype {
            NecType::Nec => TimeUnits::new(period, &GENERIC_TIMING),
            NecType::Samsung => TimeUnits::new(period, &SAMSUNG_TIMING),
        };

        Self {
            state: TransmitStateInternal::Idle,
            units,
            last_ts: 0,
            cmd: 0,
        }
    }
}

impl Transmitter for NecTransmitter {
    fn set_command<CMD: Into<u32>>(&mut self, cmd: CMD) {
        self.cmd = cmd.into();
        self.state = TransmitStateInternal::Start;
    }

    fn reset(&mut self) {
        self.cmd = 0;
        self.state = TransmitStateInternal::Idle;
        self.last_ts = 0;
    }

    fn transmit(&mut self, ts: u32) -> TransmitterState {
        use TransmitStateInternal::*;

        let tsdiff = ts.wrapping_sub(self.last_ts);

        self.state = match self.state {
            Start => {
                self.last_ts = ts;
                HeaderHigh
            }
            HeaderHigh => {
                if tsdiff >= self.units.header_high {
                    self.last_ts = ts;
                    HeaderLow
                } else {
                    HeaderHigh
                }
            }
            HeaderLow => {
                if tsdiff >= self.units.header_low {
                    self.last_ts = ts;
                    DataHigh(0)
                } else {
                    HeaderLow
                }
            }
            DataLow(32) => Done,
            DataHigh(bidx) => {
                if tsdiff >= self.units.data_high {
                    self.last_ts = ts;
                    DataLow(bidx)
                } else {
                    DataHigh(bidx)
                }
            }
            DataLow(i) => {

                let hightime = if (self.cmd & (1 << i)) != 0 {
                    self.units.one_low
                } else {
                    self.units.zero_low
                };

                if tsdiff >= hightime {
                    self.last_ts = ts;
                    DataHigh(i+1)
                } else {
                    DataLow(i)
                }
            }
            Done => Done,
            Idle => Idle,
        };


        match self.state {
            HeaderHigh | DataHigh(_) => TransmitterState::Transmit(true),
            HeaderLow | DataLow(_) => TransmitterState::Transmit(false),

            _ => TransmitterState::Idle
        }
    }
}
