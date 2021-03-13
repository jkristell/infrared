use core::marker::PhantomData;

use crate::{
    protocols::{
        nec::{NecCommand, NecCommandVariant, NecPulseDistance},
        utils::InfraRange4,
        Nec,
    },
    recv::{Error, InfraredReceiver, InfraredReceiverState, Status},
};

pub struct NecReceiverState<C = NecCommand> {
    // State
    status: InternalStatus,
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

impl<C: NecCommandVariant> InfraredReceiverState for NecReceiverState<C> {
    fn create(samplerate: u32) -> Self {
        let tols = tolerances(C::PULSE_DISTANCE);
        let ranges = InfraRange4::new(&tols, samplerate);

        NecReceiverState {
            status: InternalStatus::Init,
            bitbuf: 0,
            ranges,
            last_cmd: 0,
            cmd_type: Default::default(),
            dt_save: 0,
        }
    }

    fn reset(&mut self) {

        // Check if a command was successfully received
        let done = self.status == InternalStatus::Done;

        // Save the bitbuf as last cmd. This is used by the repeat logic
        if self.bitbuf != 0 && done {
            self.last_cmd = self.bitbuf;
        }

        self.status = InternalStatus::Init;
        self.bitbuf = 0;
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
    Err(Error),
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

impl<Cmd> InfraredReceiver for Nec<Cmd>
where
    Cmd: NecCommandVariant,
{
    type ReceiverState = NecReceiverState<Cmd>;
    type InternalStatus = InternalStatus;

    #[rustfmt::skip]
    fn event(state: &mut Self::ReceiverState, rising: bool, dt: u32) -> Self::InternalStatus {
        use InternalStatus::*;
        use PulseWidth::*;

        if rising {
            let pulsewidth = state.ranges.find::<PulseWidth>(state.dt_save + dt).unwrap_or(PulseWidth::NotAPulseWidth);

            state.status = match (state.status, pulsewidth) {
                (Init,              Sync)   => Receiving(0),
                (Init,              Repeat) => RepeatDone,
                (Init,              _)      => Init,

                (Receiving(31),     One)    => { state.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { state.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(Error::Data),

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

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        match state.status {
            InternalStatus::Done => Self::Cmd::unpack(state.bitbuf, false),
            InternalStatus::RepeatDone => Self::Cmd::unpack(state.last_cmd, true),
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
        ((t.header_high + t.header_low), 10),
        ((t.header_high + t.repeat_low), 10),
        ((t.data_high + t.data_zero_low), 5),
        ((t.data_high + t.data_one_low), 5),
    ]
}
