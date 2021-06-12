use crate::protocol::InfraredProtocol;
use crate::protocols::utils::InfraConstRange;
use crate::recv::InfraredReceiverState;
use crate::recv::{InfraredReceiver, Status};

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

impl InfraredProtocol for Denon {
    type Cmd = DenonCommand;
}

pub struct DenonReceiverState {
    state: DenonStatus,
    buf: u64,
    dt_save: u32,
    ranges: InfraConstRange<3>,
}

impl InfraredReceiverState for DenonReceiverState {
    fn create(samplerate: u32) -> Self {
        let ranges = InfraConstRange::new(&PULSELENGTHS, samplerate);

        DenonReceiverState {
            state: DenonStatus::Idle,
            buf: 0,
            dt_save: 0,
            ranges,
        }
    }

    fn reset(&mut self) {
        self.state = DenonStatus::Idle;
        self.buf = 0;
        self.dt_save = 0;
    }
}

#[derive(Debug)]
pub struct DenonCommand {
    pub bits: u64,
}

impl InfraredReceiver for Denon {
    type ReceiverState = DenonReceiverState;
    type InternalStatus = DenonStatus;

    #[rustfmt::skip]
    fn event(state: &mut Self::ReceiverState, rising: bool, dt: u32) -> DenonStatus {
        if rising {
            let pulsewidth = state.ranges.find::<PulseWidth>(state.dt_save + dt)
                .unwrap_or(PulseWidth::FAIL);

            state.state = match (state.state, pulsewidth) {
                (DenonStatus::Idle,          PulseWidth::SYNC)   => DenonStatus::Data(0),
                (DenonStatus::Idle,          _)                  => DenonStatus::Idle,
                (DenonStatus::Data(47),      PulseWidth::ZERO)   => DenonStatus::Done,
                (DenonStatus::Data(47),      PulseWidth::ONE)    => DenonStatus::Done,
                (DenonStatus::Data(idx),     PulseWidth::ZERO)   => DenonStatus::Data(idx + 1),
                (DenonStatus::Data(idx),     PulseWidth::ONE)    => { state.buf |= 1 << idx; DenonStatus::Data(idx + 1) }
                (DenonStatus::Data(_ix),     _)                  => DenonStatus::Idle,
                (DenonStatus::Done,          _)                  => DenonStatus::Done,
            };

            state.dt_save = 0;
        } else {
            state.dt_save = dt;
        }

        state.state
    }

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        if state.state == DenonStatus::Done {
            Some(DenonCommand { bits: state.buf })
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DenonStatus {
    Idle,
    Data(u8),
    Done,
}

impl Into<Status> for DenonStatus {
    fn into(self) -> Status {
        match self {
            DenonStatus::Idle => Status::Idle,
            DenonStatus::Data(_) => Status::Receiving,
            DenonStatus::Done => Status::Done,
        }
    }
}

#[derive(Debug)]
enum PulseWidth {
    SYNC,
    ZERO,
    ONE,
    FAIL,
}

impl From<u32> for PulseWidth {
    fn from(value: u32) -> Self {
        match value {
            0 => PulseWidth::SYNC,
            1 => PulseWidth::ZERO,
            2 => PulseWidth::ONE,
            _ => PulseWidth::FAIL,
        }
    }
}
