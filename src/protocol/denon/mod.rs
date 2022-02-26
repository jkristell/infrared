use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::Protocol,
    receiver::{DecoderState, DecoderStateMachine, Status},
};

#[cfg(test)]
mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;

/// Denon protocol
pub struct Denon;

impl Protocol for Denon {
    type Cmd = DenonCommand;
}

pub struct DenonReceiverState<T: InfraMonotonic> {
    state: DenonStatus,
    buf: u64,
    dt_save: T::Duration,
}

impl<T: InfraMonotonic> DecoderState for DenonReceiverState<T> {
    fn reset(&mut self) {
        self.state = DenonStatus::Idle;
        self.buf = 0;
        self.dt_save = T::ZERO_DURATION;
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DenonCommand {
    pub bits: u64,
}

impl<Time: InfraMonotonic> DecoderStateMachine<Time> for Denon {
    type State = DenonReceiverState<Time>;
    type InternalStatus = DenonStatus;

    const PULSE_LENGTHS: [u32; 8] = [
        (HEADER_HIGH + HEADER_LOW),
        (DATA_HIGH + ZERO_LOW),
        (DATA_HIGH + ONE_LOW),
        0,
        0,
        0,
        0,
        0,
    ];

    const TOLERANCE: [u32; 8] = [8, 10, 10, 0, 0, 0, 0, 0];

    fn state() -> Self::State {
        DenonReceiverState {
            state: DenonStatus::Idle,
            buf: 0,
            dt_save: Time::ZERO_DURATION,
        }
    }

    #[rustfmt::skip]
    fn new_event(state: &mut Self::State,
                 ranges: &PulseSpans<Time::Duration>,
                 rising: bool, dt: Time::Duration) -> DenonStatus {
        if rising {
            let pulsewidth = Time::find::<PulseWidth>(ranges, state.dt_save + dt)
                .unwrap_or(PulseWidth::Fail);

            state.state = match (state.state, pulsewidth) {
                (DenonStatus::Idle,          PulseWidth::Sync)   => DenonStatus::Data(0),
                (DenonStatus::Idle,          _)                  => DenonStatus::Idle,
                (DenonStatus::Data(47),      PulseWidth::Zero)   => DenonStatus::Done,
                (DenonStatus::Data(47),      PulseWidth::One)    => DenonStatus::Done,
                (DenonStatus::Data(idx),     PulseWidth::Zero)   => DenonStatus::Data(idx + 1),
                (DenonStatus::Data(idx),     PulseWidth::One)    => { state.buf |= 1 << idx; DenonStatus::Data(idx + 1) }
                (DenonStatus::Data(_ix),     _)                  => DenonStatus::Idle,
                (DenonStatus::Done,          _)                  => DenonStatus::Done,
            };

            state.dt_save = Time::ZERO_DURATION;
        } else {
            state.dt_save = dt;
        }

        state.state
    }
    fn command(state: &Self::State) -> Option<Self::Cmd> {
        if state.state == DenonStatus::Done {
            Some(DenonCommand { bits: state.buf })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DenonStatus {
    Idle,
    Data(u8),
    Done,
}

impl From<DenonStatus> for Status {
    fn from(status: DenonStatus) -> Self {
        match status {
            DenonStatus::Idle => Status::Idle,
            DenonStatus::Data(_) => Status::Receiving,
            DenonStatus::Done => Status::Done,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum PulseWidth {
    Sync,
    Zero,
    One,
    Fail,
}

impl From<usize> for PulseWidth {
    fn from(value: usize) -> Self {
        match value {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Zero,
            2 => PulseWidth::One,
            _ => PulseWidth::Fail,
        }
    }
}
