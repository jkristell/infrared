use crate::protocols::nec::NecStandard;
use crate::{
    protocols::nec::{NecCommand, NecTiming, NecVariant},
    protocols::utils::PulseWidthRange,
    recv::{Error, ReceiverSM, State},
};
use core::marker::PhantomData;

/// Generic type for Nec Receiver
pub struct Nec<N = NecStandard> {
    // State
    state: InternalState,
    // Data buffer
    pub bitbuf: u32,
    // Timing and tolerances
    ranges: PulseWidthRange<PulseWidth>,
    // Last command (used by repeat)
    lastcommand: u32,
    // The type of Nec
    nectype: PhantomData<N>,

    last_rising: u32,
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum InternalState {
    // Waiting for first pulse
    Init,
    // Receiving data
    Receiving(u32),
    // Command received
    Done,
    // Repeat command received
    RepeatDone,
    // In error state
    Err(Error),
}

impl From<InternalState> for State {
    fn from(ns: InternalState) -> Self {
        use InternalState::*;
        match ns {
            Init => State::Idle,
            Done | RepeatDone => State::Done,
            Err(e) => State::Error(e),
            _ => State::Receiving,
        }
    }
}

impl<N: NecVariant> Default for Nec<N> {
    fn default() -> Self {
        Self::with_timing(N::TIMING)
    }
}

/// Nec decoder statemachine
impl<VARIANT: NecVariant> Nec<VARIANT> {
    pub fn new() -> Self {
        let timing = VARIANT::TIMING;
        Self::with_timing(timing)
    }

    fn with_timing(timing: &NecTiming) -> Self {
        let tols = tolerances(timing);
        let ranges = PulseWidthRange::new(&tols);

        Self {
            state: InternalState::Init,
            bitbuf: 0,
            lastcommand: 0,
            nectype: PhantomData,
            ranges,
            last_rising: 0,
        }
    }
}

impl<N: NecVariant> ReceiverSM for Nec<N> {
    type Cmd = NecCommand<N>;
    type InternalState = InternalState;

    fn create() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Self::InternalState {
        use InternalState::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = self.ranges.pulsewidth(self.last_rising + dt);

            self.state = match (self.state, pulsewidth) {
                (Init,  Sync)   => Receiving(0),
                (Init,  Repeat) => RepeatDone,
                (Init,  _)      => Init,

                (Receiving(31),     One)    => { self.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { self.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(Error::Data),

                (Done,          _)  => Done,
                (RepeatDone,    _)  => RepeatDone,
                (Err(err),      _)  => Err(err),
            };

            self.last_rising = 0;
        } else {
            // Save dist
            self.last_rising = dt;
        }

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(N::cmd_from_bits(self.bitbuf))
    }

    fn reset(&mut self) {
        self.state = InternalState::Init;
        self.lastcommand = if self.bitbuf == 0 {
            self.lastcommand
        } else {
            self.bitbuf
        };
        self.bitbuf = 0;
        self.last_rising = 0;
    }
}

#[derive(Debug, Clone)]
pub enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    NotAPulseWidth = 4,
}

impl Default for PulseWidth {
    fn default() -> Self {
        PulseWidth::NotAPulseWidth
    }
}

impl From<usize> for PulseWidth {
    fn from(v: usize) -> Self {
        match v {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Repeat,
            2 => PulseWidth::Zero,
            3 => PulseWidth::One,
            _ => PulseWidth::NotAPulseWidth,
        }
    }
}

const fn tolerances(t: &NecTiming) -> [(u32, u32); 4] {
    [
        ((t.hh + t.hl), 5),
        ((t.hh + t.rl), 5),
        ((t.dh + t.zl), 10),
        ((t.dh + t.ol), 10),
    ]
}
