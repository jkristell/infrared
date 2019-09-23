use core::ops::Range;

use crate::nec::NecType;
use crate::nec::Timing;
use crate::nec::{SAMSUNG_TIMING, STANDARD_TIMING};
use crate::{Receiver, ReceiverState};

#[derive(Debug, Copy, Clone)]
/// The resulting command
pub struct NecCommand {
    addr: u16,
    cmd: u16,
}

impl NecCommand {
    pub fn new(bitbuf: u32) -> Self {
        let addr = ((bitbuf >> 24) & 0xFF) as u16;
        let cmd = ((bitbuf >> 8) & 0xFF) as u16;
        Self { addr, cmd}
    }
}

#[derive(Debug, Clone, Copy)]
/// Error when receiving
pub enum NecError {
    /// Couldn't determine the type of message
    CommandType(u32),
    /// Receiving data but failed to read bit
    Data,
}

pub type NecResult = ReceiverState<NecCommand, NecError>;

pub struct NecReceiver {
    // State
    pub state: InternalState,
    pub bitbuf: u32,
    pub prev_timestamp: u32,
    pub prev_pinval: bool,
    pub prev_state: InternalState,
    // Timing and tolerances
    pub tolerance: Tolerances,
    pub interval: u32,
    lastcommand: NecCommand,
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum InternalState {
    // Waiting for first pulse
    Init,
    // Receiving data
    Receiving(u32),
    // Done receiving
    Done(NecCommand),
    // In error state
    Error(NecError),
    // Disabled
    Disabled,
}

impl NecReceiver {
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
            state: InternalState::Init,
            prev_state: InternalState::Init,
            tolerance: tol,
            prev_timestamp: 0,
            prev_pinval: false,
            bitbuf: 0,
            interval: 0,
            lastcommand: NecCommand::new(0),
        }
    }
}

impl Receiver for NecReceiver
{
    type Cmd = NecCommand;
    type Err = NecError;

    fn sample(&mut self, pinval: bool, timestamp: u32) -> ReceiverState<NecCommand, NecError> {
        use InternalState::{
            Disabled, Done, Error as InternalError, Init, Receiving,
        };

        if pinval && self.prev_pinval != pinval {

            let mut interval = timestamp.wrapping_sub(self.prev_timestamp);

            if interval >= core::u16::MAX.into() {
                interval = 0;
            }

            self.prev_timestamp = timestamp;
            self.interval = interval;
            self.prev_state = self.state;

            let pulsewidth = self.tolerance.pulsewidth(interval);

            self.state = match (self.state, pulsewidth) {
                (Init, PulseWidth::Sync) => Receiving(31),
                (Init, PulseWidth::Repeat) => Done(self.lastcommand),
                (Init, _) => Init,

                (Receiving(0), _) => {
                    self.lastcommand = NecCommand::new(self.bitbuf);
                    Done(self.lastcommand)
                },
                (Receiving(idx), PulseWidth::One) => {
                    self.bitbuf |= 1 << idx;
                    Receiving(idx - 1)
                },
                (Receiving(idx), PulseWidth::Zero) => Receiving(idx - 1),
                (Receiving(idx), _) => InternalError(NecError::Data),

                (Done(cmd), _) => Done(cmd),
                (InternalError(err), _) => InternalError(err),
                (Disabled, _) => Disabled,
            };
        }

        self.prev_pinval = pinval;

        // Internalstate to ReceiverState
        match self.state {
            InternalState::Init => ReceiverState::Idle,
            InternalState::Done(cmd) => ReceiverState::Done(cmd),
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
        self.state = InternalState::Init;
        self.prev_timestamp = 0;
        self.prev_pinval = false;
        self.bitbuf = 0;
    }

    fn disable(&mut self) {
        self.state = InternalState::Disabled;
    }
}

#[derive(Debug)]
pub struct Tolerances {
    pub sync: Range<u32>,
    pub repeat: Range<u32>,
    pub zero: Range<u32>,
    pub one: Range<u32>,
}

pub enum PulseWidth {
    Sync,
    Repeat,
    Zero,
    One,

    NotAPulseWidth,
}

impl Tolerances {
    pub const fn from_timing(t: &Timing, samplerate: u32) -> Self {
        let per: u32 = 1000 / (samplerate / 1000);
        Tolerances {
            sync: sample_range((t.header_htime + t.header_ltime) / per, 5),
            repeat: sample_range((t.header_htime + t.repeat_ltime) / per, 5),
            zero: sample_range((t.data_htime + t.zero_ltime) / per, 5),
            one: sample_range((t.data_htime + t.one_ltime) / per, 5),
        }

    }
    pub fn pulsewidth(&self, samples: u32) -> PulseWidth {
        if self.sync.contains(&samples) {
            return PulseWidth::Sync;
        }
        if self.repeat.contains(&samples) {
            return PulseWidth::Repeat;
        }
        if self.one.contains(&samples) {
            return PulseWidth::One;
        }
        if self.zero.contains(&samples) {
            return PulseWidth::Zero;
        }
        PulseWidth::NotAPulseWidth
    }
}

const fn sample_range(units: u32, percent: u32) -> Range<u32> {
    let tol = (units * percent) / 100;
    (units - tol..units + tol)
}
