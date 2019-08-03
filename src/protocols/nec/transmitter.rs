use core::convert::Into;

use crate::nec::{Timing, NecType, STANDARD_TIMING, SAMSUNG_TIMING};
use crate::{Transmitter, TransmitterState};

enum TransmitStateInternal {
    Idle,
    Start,
    HeaderHigh,
    HeaderLow,
    DataLow(u32),
    DataHigh(u32),
    Done,
}

pub struct NecTransmitter {
    state: TransmitStateInternal,
    samples: NSamples,
    last_ts: u32,
    cmd: u32,
}

struct NSamples {
    header_high: u32,
    header_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

impl NecTransmitter {
    pub fn new(nectype: NecType, period: u32) -> Self {

        let units = match nectype {
            NecType::Standard => NSamples::new(period, &STANDARD_TIMING),
            NecType::Samsung => NSamples::new(period, &SAMSUNG_TIMING),
        };

        Self {
            state: TransmitStateInternal::Idle,
            samples: units,
            last_ts: 0,
            cmd: 0,
        }
    }
}

impl Transmitter for NecTransmitter {
    fn init<CMD: Into<u32>>(&mut self, cmd: CMD) {
        self.cmd = cmd.into();
        self.state = TransmitStateInternal::Start;
    }

    fn step(&mut self, ts: u32) -> TransmitterState {
        use TransmitStateInternal::*;

        let interval = ts.wrapping_sub(self.last_ts);

        self.state = match self.state {
            Start => {
                self.last_ts = ts;
                HeaderHigh
            }
            HeaderHigh => {
                if interval >= self.samples.header_high {
                    self.last_ts = ts;
                    HeaderLow
                } else {
                    HeaderHigh
                }
            }
            HeaderLow => {
                if interval >= self.samples.header_low {
                    self.last_ts = ts;
                    DataHigh(0)
                } else {
                    HeaderLow
                }
            }
            DataLow(32) => Done,
            DataHigh(bidx) => {
                if interval >= self.samples.data_high {
                    self.last_ts = ts;
                    DataLow(bidx)
                } else {
                    DataHigh(bidx)
                }
            }
            DataLow(i) => {
                let hightime = if (self.cmd & (1 << i)) != 0 {
                    self.samples.one_low
                } else {
                    self.samples.zero_low
                };

                if interval >= hightime {
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
            Done | Idle | Start => TransmitterState::Idle
        }
    }

    fn reset(&mut self) {
        self.cmd = 0;
        self.state = TransmitStateInternal::Idle;
        self.last_ts = 0;
    }
}

impl NSamples {
    pub const fn new(period: u32, timing: &Timing) -> Self {
        Self {
            header_high: timing.header_htime / period,
            header_low: timing.header_ltime / period,
            zero_low: timing.zero_ltime / period,
            data_high: timing.data_htime / period,
            one_low: timing.one_ltime / period,
        }
    }
}


