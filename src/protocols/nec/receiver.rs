use crate::protocols::nec::{NecCommandTrait};
use crate::{
    protocols::nec::{NecPulseDistance, NecTiming, NecCommand},
    protocols::utils::PulseWidthRange,
    recv::{Error, InfraredReceiver, Status},
};
use core::marker::PhantomData;
use crate::protocols::utils::InfraRange4;
use crate::protocolid::InfraredProtocol;
use crate::recv::InfraredReceiverState;
use crate::protocols::Nec;

pub struct NecReceiverState<C = NecCommand>  {
    // State
    status: InternalState,
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

impl<C: NecTiming> InfraredReceiverState for NecReceiverState<C> {
    fn create(samplerate: u32) -> Self {

        let tols = tolerances(C::PD);
        let ranges = InfraRange4::new(&tols, samplerate);

        NecReceiverState {
            status: InternalState::Init,
            bitbuf: 0,
            ranges,
            last_cmd: 0,
            cmd_type: Default::default(),
            dt_save: 0
        }
    }

    fn reset(&mut self) {
        self.status = InternalState::Init;
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


impl<Cmd> InfraredReceiver for Nec<Cmd>
where
    Cmd: NecCommandTrait + NecTiming,
{
    type ReceiverState = NecReceiverState<Cmd>;
    type InternalState = InternalState;

    #[rustfmt::skip]
    fn event(state: &mut Self::ReceiverState, rising: bool, dt: u32) -> Self::InternalState {
        use InternalState::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = state.ranges.find::<PulseWidth>(state.dt_save + dt).unwrap_or(PulseWidth::NotAPulseWidth);

            state.status = match (state.status, pulsewidth) {
                (Init,  Sync)   => Receiving(0),
                (Init,  Repeat) => RepeatDone,
                (Init,  _)      => Init,

                (Receiving(31),     One)    => { state.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { state.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(Error::Data),

                (Done,          _)  => Done,
                (RepeatDone,    _)  => RepeatDone,
                (Err(err),      _)  => Err(err),
            };

            state.dt_save = 0;
        } else {
            // Save
            state.dt_save = dt;
        }

        state.status
    }

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        match state.status {
            InternalState::Done => Self::Cmd::unpack(state.bitbuf, false),
            InternalState::RepeatDone => Self::Cmd::unpack(state.last_cmd, true),
            _ => None,
        }
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
