use crate::protocols::nec::{NecCommandTrait};
use crate::{
    protocols::nec::{NecPulseDistance, NecTiming, NecCommand},
    recv::{Error, InfraredReceiver, State},
};
use core::marker::PhantomData;
use crate::protocols::utils::InfraRange4;

/// Nec Receiver with Nec standard bit encoding and Standard timing
pub struct Nec<C = NecCommand> {
    // State
    state: InternalState,
    // Data buffer
    bitbuf: u32,
    // Timing and tolerances
    ranges: InfraRange4,
    // Last command (used by repeat)
    last_cmd: u32,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: u32,
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

/// Nec decoder statemachine
impl<Cmd: NecTiming + NecCommandTrait> Nec<Cmd> {

    fn new() -> Self {
        Self::with_samplerate(1_000_000)
    }

    fn with_samplerate(samplerate: u32) -> Self {
        let tols = tolerances(Cmd::PD);
        let ranges = InfraRange4::new(&tols, samplerate);

        Self {
            state: InternalState::Init,
            bitbuf: 0,
            last_cmd: 0,
            ranges,
            dt_save: 0,
            cmd_type: Default::default(),
        }
    }
}

impl<Cmd> InfraredReceiver for Nec<Cmd>
where
    Cmd: NecCommandTrait + NecTiming,
{
    type Cmd = Cmd;
    type InternalState = InternalState;

    fn create() -> Self {
        Self::new()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Self::InternalState {
        use InternalState::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = self.ranges.find::<PulseWidth>(self.dt_save + dt).unwrap_or(PulseWidth::NotAPulseWidth);

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

            self.dt_save = 0;
        } else {
            // Save
            self.dt_save = dt;
        }

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        match self.state {
            InternalState::Done => Self::Cmd::unpack(self.bitbuf, false),
            InternalState::RepeatDone => Self::Cmd::unpack(self.last_cmd, true),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.state = InternalState::Init;
        self.last_cmd = if self.bitbuf == 0 {
            self.last_cmd
        } else {
            self.bitbuf
        };
        self.bitbuf = 0;
        self.dt_save = 0;
    }
}

#[derive(Debug, Copy, Clone)]
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

const fn tolerances(t: &NecPulseDistance) -> [(u32, u32); 4] {
    [
        ((t.hh + t.hl), 5),
        ((t.hh + t.rl), 5),
        ((t.dh + t.zl), 10),
        ((t.dh + t.ol), 10),
    ]
}
