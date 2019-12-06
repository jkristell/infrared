use crate::{
    Transmitter, TransmitterState,
    nec::{
        NecCommand, NecTiming, NecVariant,
    }
};

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
    hh: u32,
    hl: u32,
    data: u32,
    zero: u32,
    one: u32,
}

impl<NECTYPE: NecVariant> NecTypeTransmitter<NECTYPE> {
    pub fn new(samplerate: u32) -> Self {
        let period: u32 = (1 * 1000) / (samplerate / 1000);

        let samples = NSamples::new(period, &NECTYPE::TIMING);
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
where
    NECTYPE: NecVariant,
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
                if interval >= self.samples.hh {
                    self.last_ts = ts;
                    HeaderLow
                } else {
                    HeaderHigh
                }
            }
            HeaderLow => {
                if interval >= self.samples.hl {
                    self.last_ts = ts;
                    DataHigh(0)
                } else {
                    HeaderLow
                }
            }

            DataHigh(bidx) => {
                if interval >= self.samples.data {
                    self.last_ts = ts;
                    DataLow(bidx)
                } else {
                    DataHigh(bidx)
                }
            }
            DataLow(32) => Done,
            DataLow(bidx) => {
                let samples = if (self.cmd & (1 << bidx)) != 0 {
                    self.samples.one
                } else {
                    self.samples.zero
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
impl<NECTYPE: NecVariant> crate::PwmTransmitter<NecCommand> for NecTypeTransmitter<NECTYPE> {}

impl NSamples {
    pub const fn new(period: u32, pulsedistance: &NecTiming) -> Self {
        Self {
            hh: pulsedistance.hh / period,
            hl: pulsedistance.hl / period,
            zero: pulsedistance.zl / period,
            data: pulsedistance.dh / period,
            one: pulsedistance.ol / period,
        }
    }
}
