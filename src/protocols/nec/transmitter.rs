use core::convert::Into;

use crate::nec::{Timing};
use crate::{Transmitter, TransmitterState};
use crate::protocols::nec::NecTypeTrait;

enum TransmitStateInternal {
    Idle,
    Start,
    HeaderHigh,
    HeaderLow,
    DataLow(u32),
    DataHigh(u32),
    Done,
}

pub struct NecTypeTransmitter<NECTYPE> {
    state: TransmitStateInternal,
    samples: NSamples,
    last_ts: u32,
    cmd: u32,
    nectype: core::marker::PhantomData<NECTYPE>,
}

struct NSamples {
    header_high: u32,
    header_low: u32,
    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

impl<NECTYPE: NecTypeTrait> NecTypeTransmitter<NECTYPE> {
    pub fn new(period: u32) -> Self {

        let units = NSamples::new(period, &NECTYPE::TIMING);

        Self {
            state: TransmitStateInternal::Idle,
            samples: units,
            last_ts: 0,
            cmd: 0,
            nectype: core::marker::PhantomData,
        }
    }
}

impl<NECTYPE> Transmitter for NecTypeTransmitter<NECTYPE> {
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

            DataHigh(bidx) => {
                if interval >= self.samples.data_high {
                    self.last_ts = ts;
                    DataLow(bidx)
                } else {
                    DataHigh(bidx)
                }
            }
            DataLow(32) => Done,
            DataLow(bidx) => {
                let samples = if (self.cmd & (1 << bidx)) != 0 {
                    self.samples.one_low
                } else {
                    self.samples.zero_low
                };

                if interval >= samples {
                    self.last_ts = ts;
                    DataHigh(bidx + 1)
                } else {
                    DataLow(bidx)
                }
            }
            Done => Done,
            Idle => Idle,
        };

        match self.state {
            HeaderHigh | DataHigh(_) => TransmitterState::Transmit(true),
            HeaderLow | DataLow(_) => TransmitterState::Transmit(false),
            Done | Idle | Start => TransmitterState::Idle,
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
            header_high: timing.header_high / period,
            header_low: timing.header_low / period,
            zero_low: timing.zero_low / period,
            data_high: timing.data_high / period,
            one_low: timing.one_low / period,
        }
    }
}
