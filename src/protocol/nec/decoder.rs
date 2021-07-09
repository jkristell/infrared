use core::marker::PhantomData;

use crate::protocol::utils::InfraConstRange;
use crate::receiver::ConstDecodeStateMachine;
use crate::{
    protocol::{
        nec::{NecCommand, NecCommandVariant, NecPulseDistance},
        Nec,
    },
    receiver::{DecoderState, DecoderStateMachine, DecodingError, Status},
};

pub struct NecReceiverState<C = NecCommand> {
    // State
    status: InternalStatus,
    // Data buffer
    bitbuf: u32,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: usize,
}

impl<C: NecCommandVariant> DecoderState for NecReceiverState<C> {
    fn reset(&mut self) {
        self.status = InternalStatus::Init;
        self.dt_save = 0;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
// Internal receiver state
pub enum InternalStatus {
    // Waiting for first pulse
    Init,
    // Receiving data
    Receiving(u32),
    // Command received
    Done,
    // Repeat command received
    RepeatDone,
    // In error state
    Err(DecodingError),
}

impl From<InternalStatus> for Status {
    fn from(ns: InternalStatus) -> Self {
        use InternalStatus::*;
        match ns {
            Init => Status::Idle,
            Done | RepeatDone => Status::Done,
            Err(e) => Status::Error(e),
            _ => Status::Receiving,
        }
    }
}

impl<Cmd> DecoderStateMachine for Nec<Cmd>
where
    Cmd: NecCommandVariant,
{
    type State = NecReceiverState<Cmd>;
    type RangeData = InfraConstRange<6>;
    type InternalStatus = InternalStatus;

    fn state() -> Self::State {
        NecReceiverState {
            status: InternalStatus::Init,
            bitbuf: 0,
            cmd_type: Default::default(),
            dt_save: 0,
        }
    }
    fn ranges(resolution: usize) -> Self::RangeData {
        let tols = tolerances(Cmd::PULSE_DISTANCE);
        InfraConstRange::new(&tols, resolution)
    }

    #[rustfmt::skip]
    fn event_full(state: &mut Self::State, ranges: &Self::RangeData, rising: bool, dt: usize) -> Self::InternalStatus {
        use InternalStatus::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = ranges.find::<PulseWidth>(state.dt_save + dt).unwrap_or(PulseWidth::NotAPulseWidth);

            state.status = match (state.status, pulsewidth) {
                (Init,              Sync)   => { state.bitbuf = 0; Receiving(0) },
                (Init,              Repeat) => RepeatDone,
                (Init,              _)      => Init,

                (Receiving(31),     One)    => { state.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { state.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(DecodingError::Data),

                (Done,              _)      => Done,
                (RepeatDone,        _)      => RepeatDone,
                (Err(err),          _)      => Err(err),
            };

            state.dt_save = 0;
        } else {
            // Save
            state.dt_save = dt;
        }

        state.status
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        match state.status {
            InternalStatus::Done => Self::Cmd::unpack(state.bitbuf, false),
            InternalStatus::RepeatDone => Self::Cmd::unpack(state.bitbuf, true),
            _ => None,
        }
    }
}

impl<Cmd: NecCommandVariant, const R: usize> ConstDecodeStateMachine<R> for Nec<Cmd> {
    const RANGES: Self::RangeData = InfraConstRange::new(&tolerances(Cmd::PULSE_DISTANCE), R);
}

#[derive(Debug, Copy, Clone)]
pub enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    NotAPulseWidth = 4,
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

const fn tolerances(t: &NecPulseDistance) -> [(usize, usize); 4] {
    [
        ((t.header_high + t.header_low), 10),
        ((t.header_high + t.repeat_low), 10),
        ((t.data_high + t.data_zero_low), 5),
        ((t.data_high + t.data_one_low), 5),
    ]
}
