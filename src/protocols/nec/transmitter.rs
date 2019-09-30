use crate::nec::{Pulsedistance, NecCommand};
use crate::prelude::*;
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
        let samples = NSamples::new(period, &NECTYPE::PULSEDISTANCE);
        Self {
            state: TransmitStateInternal::Idle,
            samples,
            last_ts: 0,
            cmd: 0,
            nectype: core::marker::PhantomData,
        }
    }
}

impl<NECTYPE> Transmitter<NecCommand> for NecTypeTransmitter<NECTYPE>
    where NECTYPE: NecTypeTrait,
{
    fn load(&mut self, cmd: NecCommand) {
        self.cmd = NECTYPE::encode_command(cmd);
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

#[cfg(feature = "embedded-hal")]
impl<NECTYPE: NecTypeTrait> PwmTransmitter<NecCommand> for NecTypeTransmitter<NECTYPE> {

    fn pwmstep<DUTY>(&mut self, ts: u32, pwm: &mut impl embedded_hal::PwmPin<Duty=DUTY>) -> TransmitterState {

        let state = self.step(ts);
        match state {
            TransmitterState::Transmit(true) => pwm.enable(),
            _ => pwm.disable(),
        }
        state
    }
}

impl NSamples {
    pub const fn new(period: u32, pulsedistance: &Pulsedistance) -> Self {
        Self {
            header_high: pulsedistance.header_high / period,
            header_low: pulsedistance.header_low / period,
            zero_low: pulsedistance.zero_low / period,
            data_high: pulsedistance.data_high / period,
            one_low: pulsedistance.one_low / period,
        }
    }
}
