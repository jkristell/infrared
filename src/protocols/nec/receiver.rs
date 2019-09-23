use core::convert::From;
use core::ops::Range;

use crate::nec::NecType;
use crate::nec::Timing;
use crate::nec::{SAMSUNG_TIMING, STANDARD_TIMING};
use crate::{Receiver, ReceiverState};

#[derive(Debug, Clone)]
/// The Command types
pub enum NecCommand<T>
where
    T: Clone + From<u32>,
{
    /// A Repeat
    Repeat,
    /// A Command With device address and action
    Payload(T),
}

#[derive(Debug, Clone, Copy)]
/// Error when receiving
pub enum NecError {
    /// Couldn't determine the type of message
    CommandType(u32),
    /// Receiving data but failed to read bit
    Data,
}

pub type NecResult<T> = ReceiverState<NecCommand<T>, NecError>;

pub struct NecReceiver<T: Clone + From<u32>> {
    // State
    state: InternalState<T>,
    bitbuf: u32,
    bitbuf_idx: u32,
    prev_timestamp: u32,
    // Timing and tolerances
    tolerance: Tolerances,
}

#[derive(Clone)]
// Internal receiver state
enum InternalState<T: Clone + From<u32>> {
    // Waiting for first edge
    Idle,
    // Determining the type of message
    HeaderHigh,
    HeaderLow,
    // Receiving data
    Receiving(u32),
    // Done receiving
    Done(NecCommand<T>),
    // In error state
    Error(NecError),
    // Disabled
    Disabled,
}

impl<T> NecReceiver<T>
where
    T: Clone + From<u32>,
{
    pub fn new(variant: NecType, samplerate: u32) -> Self {
        let timing = match variant {
            NecType::Standard => &STANDARD_TIMING,
            NecType::Samsung => &SAMSUNG_TIMING,
        };

        Self::new_from_timing(samplerate, timing)
    }

    fn new_from_timing(samplerate: u32, timing: &Timing) -> Self {
        let tol = Tolerances::from_timing(timing, samplerate);
        Self {
            state: InternalState::Idle,
            tolerance: tol,
            prev_timestamp: 0,
            bitbuf_idx: 0,
            bitbuf: 0,
        }
    }
}

impl<T> Receiver for NecReceiver<T>
where
    T: Clone + From<u32>,
{
    type Cmd = NecCommand<T>;
    type Err = NecError;

    fn sample(&mut self, rising: bool, timestamp: u32) -> ReceiverState<NecCommand<T>, NecError> {
        use InternalState::{
            Disabled, Done, Error as InternalError, HeaderHigh, HeaderLow, Idle, Receiving,
        };

        let interval = timestamp.wrapping_sub(self.prev_timestamp);
        self.prev_timestamp = timestamp;

        self.state = match (self.state.clone(), rising) {
            (Idle, true) => HeaderHigh,
            (Idle, false) => Idle,

            (HeaderHigh, true) => unreachable!(),
            (HeaderHigh, false) => {
                if self.tolerance.is_sync_high(interval) {
                    HeaderLow
                } else {
                    InternalError(NecError::CommandType(interval))
                }
            }

            (HeaderLow, false) => unreachable!(),
            (HeaderLow, true) => {
                if self.tolerance.is_sync_low(interval) {
                    Receiving(0)
                } else if self.tolerance.is_repeat(interval) {
                    Done(NecCommand::Repeat)
                } else {
                    InternalError(NecError::CommandType(interval))
                }
            }

            (Receiving(_saved), false) => Receiving(interval),
            (Receiving(saved), true) => {
                let tsdiff = interval + saved;

                if let Some(one) = self.tolerance.is_value(tsdiff) {
                    if one {
                        self.bitbuf |= 1 << self.bitbuf_idx;
                    }
                    self.bitbuf_idx += 1;
                    if self.bitbuf_idx == 32 {
                        Done(NecCommand::Payload(self.bitbuf.into()))
                    } else {
                        Receiving(0)
                    }
                } else {
                    InternalError(NecError::Data)
                }
            }
            (Done(_), _) => Disabled,
            (InternalError(_), _) => Disabled,
            (Disabled, _) => Disabled,
        };

        // Internalstate to ReceiverState
        match self.state.clone() {
            InternalState::Idle => ReceiverState::Idle,
            InternalState::Done(cmd) => {
                self.reset();
                ReceiverState::Done(cmd)
            }
            InternalState::Error(e) => ReceiverState::Error(e),
            InternalState::Disabled => ReceiverState::Disabled,
            _ => ReceiverState::Receiving,
        }
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err> {
        unimplemented!()
    }

    fn sample_edge_delta(&mut self, rising: bool, sampledelta: u16) -> ReceiverState<Self::Cmd, Self::Err> {
        unimplemented!()
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
    pub const fn from_timing(t: &Timing, samplerate: u32) -> Self {
        let per: u32 = 1000 / (samplerate / 1000);
        Tolerances {
            sync_high: unit_range(t.header_htime / per, 5),
            sync_low: unit_range(t.header_ltime / per, 5),
            repeat: unit_range(t.repeat_ltime / per, 5),
            zero: unit_range((t.data_htime + t.zero_ltime) / per, 15),
            one: unit_range((t.data_htime + t.one_ltime) / per, 15),
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
