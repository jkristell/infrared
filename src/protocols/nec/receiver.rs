use crate::protocols::nec::{StandardTiming, NecCommandTrait};
use crate::{
    protocols::nec::{NecRaw, NecPulselengths, NecTiming},
    protocols::utils::PulseWidthRange,
    recv::{Error, ReceiverSM, State},
};
use core::marker::PhantomData;
use crate::protocols::nec::cmds::{NecCommand};

/// Generic type for Nec Receiver
pub struct Nec<C = NecCommand, T = StandardTiming> {
    // State
    state: InternalState,
    // Data buffer
    pub bitbuf: u32,
    // Timing and tolerances
    ranges: PulseWidthRange<PulseWidth>,
    // Last command (used by repeat)
    lastcommand: u32,
    // The type of Nec
    nectype: PhantomData<T>,
    // Nec Command type

    neccmd: PhantomData<C>,

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

impl<Cmd, Timing: NecTiming> Default for Nec<Cmd, Timing> {
    fn default() -> Self {
        Self::with_timing(Timing::PL)
    }
}

/// Nec decoder statemachine
impl<Cmd, Timing: NecTiming> Nec<Cmd, Timing> {
    pub fn new() -> Self {
        let timing = Timing::PL;
        Self::with_timing(timing)
    }

    fn with_timing(timing: &NecPulselengths) -> Self {
        let tols = tolerances(timing);
        let ranges = PulseWidthRange::new(&tols);

        Self {
            state: InternalState::Init,
            bitbuf: 0,
            lastcommand: 0,
            nectype: PhantomData,

            ranges,
            last_rising: 0,
            neccmd: Default::default()
        }
    }
}

impl<Cmd, Timing> ReceiverSM for Nec<Cmd, Timing>
where
    Cmd: NecCommandTrait<Timing>,
    Timing: NecTiming,
{

    type Cmd = Cmd;
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

        match self.state {
            InternalState::Done => {
                let rawcmd = NecRaw { bits: self.bitbuf, };
                let cmd = Self::Cmd::unpack(rawcmd, false)?;
                Some(cmd)
            }
            InternalState::RepeatDone => {
                let rawcmd = NecRaw { bits: self.lastcommand, };
                let cmd = Self::Cmd::unpack(rawcmd, true)?;
                Some(cmd)
            }
            _ => None,
        }
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

const fn tolerances(t: &NecPulselengths) -> [(u32, u32); 4] {
    [
        ((t.hh + t.hl), 5),
        ((t.hh + t.rl), 5),
        ((t.dh + t.zl), 10),
        ((t.dh + t.ol), 10),
    ]
}
