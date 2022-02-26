use core::marker::PhantomData;

use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{
        nec::{NecCommand, NecCommandVariant},
        Nec,
    },
    receiver::{DecoderState, DecoderStateMachine, DecodingError, Status},
};

pub struct NecReceiverState<Mono: InfraMonotonic, C = NecCommand> {
    // State
    status: InternalStatus,
    // Data buffer
    bitbuf: u32,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: Mono::Duration,
    dt_int: u32,
}

impl<C: NecCommandVariant, Mono: InfraMonotonic> DecoderState for NecReceiverState<Mono, C> {
    fn reset(&mut self) {
        self.status = InternalStatus::Init;
        self.dt_save = Mono::ZERO_DURATION;
        self.dt_int = 0;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl<Cmd, Time> DecoderStateMachine<Time> for Nec<Cmd>
where
    Cmd: NecCommandVariant,
    Time: InfraMonotonic,
{
    type State = NecReceiverState<Time, Cmd>;
    type InternalStatus = InternalStatus;

    const PULSE_LENGTHS: [u32; 8] = [
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.header_low,
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.repeat_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_zero_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_one_low,
        0,
        0,
        0,
        0,
    ];

    const TOLERANCE: [u32; 8] = [7, 7, 5, 5, 0, 0, 0, 0];

    fn state() -> Self::State {
        NecReceiverState {
            status: InternalStatus::Init,
            bitbuf: 0,
            cmd_type: Default::default(),
            dt_save: Time::ZERO_DURATION,
            dt_int: 0,
        }
    }

    #[rustfmt::skip]
    fn new_event(
        state: &mut Self::State,
        spans: &PulseSpans<Time::Duration>,
        rising: bool,
        dur: Time::Duration,
    ) -> Self::InternalStatus {

        use InternalStatus::*;
        use PulseWidth::*;

        if rising {

            let total_duration = dur + state.dt_save;

            let pulsewidth = Time::find::<PulseWidth>(spans, total_duration)
                .unwrap_or(PulseWidth::Invalid);


            let status = match (state.status, pulsewidth) {
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

            trace!(
                "State(prev, new): ({:?}, {:?}) pulsewidth: {:?}",
                state.status,
                status,
                pulsewidth
            );

            state.status = status;

            state.dt_save = Time::ZERO_DURATION;
        } else {
            // Save
            state.dt_save = dur;
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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    Invalid = 4,
}

impl From<usize> for PulseWidth {
    fn from(v: usize) -> Self {
        match v {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Repeat,
            2 => PulseWidth::Zero,
            3 => PulseWidth::One,
            _ => PulseWidth::Invalid,
        }
    }
}
