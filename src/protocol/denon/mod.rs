use crate::{
    protocol::{utils::InfraConstRange, Protocol},
    receiver::{ConstDecodeStateMachine, DecoderState, DecoderStateMachine, Status},
};

#[cfg(test)]
mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;

const PULSELENGTHS: [(u32, u32); 3] = [
    ((HEADER_HIGH + HEADER_LOW), 5),
    ((DATA_HIGH + ZERO_LOW), 10),
    ((DATA_HIGH + ONE_LOW), 10),
];

/// Denon protocol
pub struct Denon;

impl Protocol for Denon {
    type Cmd = DenonCommand;
}

pub struct DenonReceiverState {
    state: DenonStatus,
    buf: u64,
    dt_save: u32,
}

impl DecoderState for DenonReceiverState {
    fn reset(&mut self) {
        self.state = DenonStatus::Idle;
        self.buf = 0;
        self.dt_save = 0;
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DenonCommand {
    pub bits: u64,
}

impl DecoderStateMachine for Denon {
    type State = DenonReceiverState;
    type InternalStatus = DenonStatus;
    type RangeData = InfraConstRange<3>;

    fn state() -> Self::State {
        DenonReceiverState {
            state: DenonStatus::Idle,
            buf: 0,
            dt_save: 0,
        }
    }

    fn ranges(resolution: u32) -> Self::RangeData {
        InfraConstRange::<3>::new(&PULSELENGTHS, resolution)
    }

    #[rustfmt::skip]
    fn event_full(state: &mut Self::State, ranges: &Self::RangeData, rising: bool, dt: u32) -> DenonStatus {
        if rising {
            let pulsewidth = ranges.find::<PulseWidth>(state.dt_save + dt)
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

            state.dt_save = 0;
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

impl<const R: u32> ConstDecodeStateMachine<R> for Denon {
    const RANGES: Self::RangeData = InfraConstRange::<3>::new(&PULSELENGTHS, R);
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
