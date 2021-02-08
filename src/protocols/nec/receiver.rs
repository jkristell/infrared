use crate::protocols::nec::{NecCommandTrait, StandardTiming};
use crate::{
    protocols::nec::{NecPulseDistance, NecTiming, NecCommand},
    protocols::utils::PulseWidthRange,
    recv::{Error, InfraredReceiver, Status},
};
use core::marker::PhantomData;
use crate::protocolid::InfraredProtocol;
use crate::recv::InfraredReceiverState;

/// Nec Receiver with Nec standard bit encoding and Standard timing
pub struct Nec<C = NecCommand, T = StandardTiming> {
    // The type of Nec
    timing: PhantomData<T>,
    // Nec Command type
    cmd_type: PhantomData<C>,
}

pub struct NecReceiverState<C = NecCommand, T = StandardTiming>  {
    // State
    status: InternalState,
    // Data buffer
    bitbuf: u32,
    // Timing and tolerances
    ranges: PulseWidthRange<PulseWidth>,
    // Last command (used by repeat)
    last_cmd: u32,
    // The type of Nec
    timing: PhantomData<T>,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: u32,
}

impl<C, T> InfraredReceiverState for NecReceiverState<C, T> {
    fn command(&self) -> Option<Self::Cmd> {
        match state.status {
            InternalState::Done => Self::Cmd::unpack(self.bitbuf, false),
            InternalState::RepeatDone => Self::Cmd::unpack(self.last_cmd, true),
            _ => None,
        }
    }

    fn reset(&mut self) {
        state.status = InternalState::Init;
        self.last_cmd = if self.bitbuf == 0 {
            self.last_cmd
        } else {
            self.bitbuf
        };
        self.bitbuf = 0;
        self.dt_save = 0;
    }

}

impl<Cmd, Timing: NecTiming> NecReceiverState<Cmd, Timing> {
    pub fn new() -> Self {
        let timing = Timing::PL;
        Self::with_timing(timing)
    }

    fn with_timing(timing: &NecPulseDistance) -> Self {
        let tols = tolerances(timing);
        let ranges = PulseWidthRange::new(&tols);

        Self {
            status: InternalState::Init,
            bitbuf: 0,
            last_cmd: 0,
            timing: PhantomData,

            ranges,
            dt_save: 0,
            cmd_type: Default::default(),
        }
    }
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

impl From<InternalState> for Status {
    fn from(ns: InternalState) -> Self {
        use InternalState::*;
        match ns {
            Init => Status::Idle,
            Done | RepeatDone => Status::Done,
            Err(e) => Status::Error(e),
            _ => Status::Receiving,
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

    fn with_timing(timing: &NecPulseDistance) -> Self {
        Self {
            timing: PhantomData,
            cmd_type: Default::default(),
        }
    }
}

impl<Cmd, Timing> InfraredReceiver for Nec<Cmd, Timing>
where
    Cmd: NecCommandTrait<Timing>,
    Timing: NecTiming,
{
    type Cmd = Cmd;
    type ReceiverState = NecReceiverState<Cmd, Timing>;
    type InternalState = InternalState;

    fn create_receiver() -> Self {
        Self::default()
    }

    fn create_receiver_state() -> Self::ReceiverState {
        NecReceiverState::new()
    }

    #[rustfmt::skip]
    fn event(&mut self, s: &mut Self::ReceiverState, rising: bool, dt: u32) -> Self::InternalState {
        use InternalState::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = s.ranges.pulsewidth(s.dt_save + dt);

            self.state = match (s.status, pulsewidth) {
                (Init,  Sync)   => Receiving(0),
                (Init,  Repeat) => RepeatDone,
                (Init,  _)      => Init,

                (Receiving(31),     One)    => { s.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { s.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(Error::Data),

                (Done,          _)  => Done,
                (RepeatDone,    _)  => RepeatDone,
                (Err(err),      _)  => Err(err),
            };

            s.dt_save = 0;
        } else {
            // Save
            s.dt_save = dt;
        }

        s.status
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
