use crate::{Receiver, State};
use core::ops::Range;

// NEC Header
//
// _/'''''''''\_____ DATA
//  |--- 9 ---| 4.5 |

// Samsung TV Header
//
//_/'''''\_____
// | 4.5 | 4.5 |

pub struct Timing {
    header_high: u32,
    header_low: u32,
    repeat_low: u32,
    one: u32,
    zero: u32,
}

const GENERIC_TIMING: Timing = Timing {
    header_high: 9000,
    header_low: 4500,
    repeat_low: 2250,
    one: 2250,
    zero: 1250,
};

const SAMSUNG_TIMING: Timing = Timing {
    header_high: 4500,
    header_low: 4500,
    repeat_low: 2250,
    one: 2250,
    zero: 1150,
};

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

#[derive(Debug, Clone, Copy)]
/// Error when receiving
pub enum Error {
    /// Couldn't determine the type of message
    CommandType(u32),
    /// Receiving data but failed to read bit
    Data,
}

pub type NecResult = State<NecCmd, Error>;

pub struct NecReceiver {
    // State
    state: InternalState,
    bitbuf: u32,
    bitbuf_idx: u32,
    prev_timestamp: u32,
    // Timing
    generic: Tolerances,
    samsung: Tolerances,
}

/// Receiver state
#[derive(Clone, Copy)]
enum InternalState {
    /// Waiting for first edge
    Idle,
    /// Determining the type of message
    HeaderHigh,
    HeaderLow,
    /// Receiving data
    Receiving(u32),
    /// Done receiving
    Done(NecCmd),
    /// In error state
    Error(Error),
    /// Disabled
    Disabled,
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

    pub fn address16(&self) -> u16 {
        (self.frame & 0xFFFF) as u16
    }

    pub fn command(&self) -> u8 {
        (((self.frame >> 16) & 0xFF) as u8)
    }
}

impl NecReceiver {
    pub const fn new(freq: u32) -> Self {
        let generic = Tolerances::from_timing(&GENERIC_TIMING, freq);
        let samsung = Tolerances::from_timing(&SAMSUNG_TIMING, freq);

        Self {
            state: InternalState::Idle,
            generic,
            samsung,
            prev_timestamp: 0,
            bitbuf_idx: 0,
            bitbuf: 0,
        }
    }
}

impl Receiver<NecCmd, Error> for NecReceiver {
    fn event(&mut self, rising: bool, timestamp: u32) -> State<NecCmd, Error> {
        use InternalState::{
            Disabled, Done, Error as InternalError, HeaderHigh, HeaderLow, Idle, Receiving,
        };

        // Distance between positive edges
        let tsdiff = timestamp.wrapping_sub(self.prev_timestamp);
        self.prev_timestamp = timestamp;

        self.state = match (self.state, rising) {
            (Idle, true) => HeaderHigh,
            (Idle, false) => Idle,

            (HeaderHigh, true) => unreachable!(),
            (HeaderHigh, false) => {
                if self.generic.is_sync_high(tsdiff) {
                    HeaderLow
                } else if self.samsung.is_sync_high(tsdiff) {
                    HeaderLow
                } else {
                    InternalError(Error::CommandType(tsdiff))
                }
            }

            (HeaderLow, false) => unreachable!(),
            (HeaderLow, true) => {
                if self.generic.is_sync_low(tsdiff) {
                    Receiving(0)
                } else if self.samsung.is_sync_high(tsdiff) {
                    Receiving(0)
                } else if self.generic.is_repeat(tsdiff) {
                    Done(NecCmd::Repeat)
                } else {
                    InternalError(Error::CommandType(tsdiff))
                }
            }

            (Receiving(_saved), false) => Receiving(tsdiff),
            (Receiving(saved), true) => {
                let tsdiff = tsdiff + saved;

                if let Some(one) = self.generic.is_value(tsdiff) {
                    if one {
                        self.bitbuf |= 1 << self.bitbuf_idx;
                    }
                    self.bitbuf_idx += 1;
                    if self.bitbuf_idx == 32 {
                        Done(NecCmd::Command(Button { frame: self.bitbuf }))
                    } else {
                        Receiving(0)
                    }
                } else {
                    InternalError(Error::Data)
                }
            }
            (Done(_), _) => Disabled,
            (InternalError(_), _) => Disabled,
            (Disabled, _) => Disabled,
        };

        match self.state {
            InternalState::Idle => State::Idle,
            InternalState::Done(cmd) => {
                self.reset();
                State::Done(cmd)
            }
            InternalState::Error(e) => State::Err(e),
            _ => State::Receiving,
        }
    }

    fn reset(&mut self) {
        self.state = InternalState::Idle;
        self.prev_timestamp = 0;
        self.bitbuf_idx = 0;
        self.bitbuf = 0;
    }

    fn disable(&mut self) {
        self.state = InternalState::Disabled;
    }
}

#[derive(Debug)]
pub struct Tolerances {
    pub sync_high: Range<u32>,
    pub sync_low: Range<u32>,
    pub repeat: Range<u32>,
    pub zero: Range<u32>,
    pub one: Range<u32>,
}

impl Tolerances {
    pub const fn from_timing(t: &Timing, freq: u32) -> Self {
        let per: u32 = (1 * 1000) / (freq / 1000);
        Tolerances {
            sync_high: unit_range(t.header_high / per, 5),
            sync_low: unit_range(t.header_low / per, 5),
            repeat: unit_range(t.repeat_low / per, 5),
            zero: unit_range(t.zero / per, 15),
            one: unit_range(t.one / per, 15),
        }
    }

    pub fn is_sync_high(&self, units: u32) -> bool {
        self.sync_high.contains(&units)
    }

    pub fn is_sync_low(&self, units: u32) -> bool {
        self.sync_low.contains(&units)
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
    (units - tol..units + tol)
}
