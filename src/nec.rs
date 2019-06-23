use crate::Receiver;
use core::ops::Range;

pub type NecResult = Result<Option<NecCmd>, Error>;

#[derive(Debug, Clone, Copy)]
/// Error when receiving
pub enum Error {
    /// Couldn't determine the type of message
    CommandType,
    /// Receiving data but failed to read bit
    Data,
}

/// Receiver state
enum State {
    /// Waiting for first edge
    Idle,
    /// Determine the type of message
    Detect,
    /// Capturing
    Data,
    /// Done
    Done(NecCmd),
    /// Error
    Error(Error),
    /// Disable
    Disabled,
}

#[derive(Clone, Copy)]
/// The Command types
pub enum NecCmd {
    Repeat,
    Command(Button),
}

#[derive(Clone, Copy)]
pub struct Button {
    pub frame: u32,
}

impl Button {
    pub fn verify(&self) -> bool {
        let frame = self.frame;
        let addr0 = (frame & 0xFF) as u8;
        let addr1 = ((frame >> 8) & 0xFF) as u8;
        let cmd0 = ((frame >> 16) & 0xFF) as u8;
        let cmd1 = ((frame >> 24) & 0xFF) as u8;

        addr0 ^ addr1 == 0xFF && cmd0 ^ cmd1 == 0xFF
    }

    pub fn address(&self) -> u8 {
        (self.frame & 0xFF) as u8
    }

    pub fn command(&self) -> u8 {
        (((self.frame >> 16) & 0xFF) as u8)
    }
}


pub struct NecReceiver {
    state: State,
    pub tolerances: Tolerances,
    bitbuf: u32,
    bitbuf_idx: u32,
    prev_timestamp: u32,
}

impl NecReceiver {
    pub const fn new(timer_freq: u32) -> Self {
        Self::new_with_tolerance(Tolerances::from_freq(timer_freq))
    }

    pub const fn new_with_tolerance(tolerances: Tolerances) -> Self {
        Self {
            state: State::Idle,
            tolerances,
            prev_timestamp: 0,
            bitbuf_idx: 0,
            bitbuf: 0,
        }
    }
}

impl Receiver<NecCmd, Error> for NecReceiver {
    fn event(&mut self, timestamp: u32) -> NecResult {

        // Distance between positive edges
        let ts_diff = timestamp.wrapping_sub(self.prev_timestamp);
        self.prev_timestamp = timestamp;

        self.state = match self.state {
            State::Idle => {
                // Got our first event.
                State::Detect
            }

            State::Detect => {
                if self.tolerances.is_sync(ts_diff) {
                    State::Data
                } else if self.tolerances.is_repeat(ts_diff) {
                    State::Done(NecCmd::Repeat)
                } else {
                    State::Error(Error::CommandType)
                }
            }

            State::Data => {
                if let Some(one) = self.tolerances.is_value(ts_diff) {
                    if one {
                        self.bitbuf |= 1 << self.bitbuf_idx;
                    }
                    self.bitbuf_idx += 1;
                    if self.bitbuf_idx == 32 {
                        State::Done(NecCmd::Command(Button {frame: self.bitbuf }))
                    } else {
                        State::Data
                    }
                } else {
                    State::Error(Error::Data)
                }
            }
            State::Done(_) => State::Disabled,
            State::Error(_) => State::Disabled,
            State::Disabled => State::Disabled,
        };


        match self.state {
            State::Done(cmd) => {
                self.reset();
                Ok(Some(cmd))
            },
            State::Error(e) => Err(e),
            _ => Ok(None),
        }
    }

    fn reset(&mut self) {
        self.state = State::Idle;
        self.prev_timestamp = 0;
        self.bitbuf_idx = 0;
        self.bitbuf = 0;
    }

    fn disable(&mut self) {
        self.state = State::Disabled;
    }
}


#[derive(Debug)]
pub struct Tolerances {
    pub sync: Range<u32>,
    pub repeat: Range<u32>,
    pub zero: Range<u32>,
    pub one: Range<u32>,
}

impl Tolerances {
    pub const fn from_freq(timer_freq: u32) -> Self {
        let period_us: u32 = (1 * 1000) / (timer_freq / 1000);
        Tolerances::from_period_us(period_us)
    }

    pub const fn from_period_us(period: u32) -> Self {
        // Values in us
        let sync_units = 13500 / period;
        let repeat_units = 11250 / period;
        let zero_units = 1250 / period;
        let one_units = 2250 / period;

        Tolerances {
            sync: unit_range(sync_units, 5),
            repeat: unit_range(repeat_units, 5),
            zero: unit_range(zero_units, 15),
            one: unit_range(one_units, 15),
        }
    }

    pub fn is_sync(&self, tsd: u32) -> bool {
        self.sync.contains(&tsd)
    }

    pub fn is_repeat(&self, tsd: u32) -> bool {
        self.repeat.contains(&tsd)
    }

    pub fn is_value(&self, tsd: u32) -> Option<bool> {
        if self.is_zero(tsd) {
            return Some(false);
        }
        if self.is_one(tsd) {
            return Some(true);
        }
        None
    }

    pub fn is_zero(&self, tsd: u32) -> bool {
        self.zero.contains(&tsd)
    }

    pub fn is_one(&self, tsd: u32) -> bool {
        self.one.contains(&tsd)
    }
}

const fn unit_range(units: u32, percent: u32) -> Range<u32> {
    let tol = (units * percent) / 100;
    (units - tol .. units + tol)
}

