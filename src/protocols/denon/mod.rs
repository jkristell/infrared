use crate::{
    protocols::utils::PulseWidthRange,
    recv::{InfraredReceiver, Status},
};
use crate::protocolid::InfraredProtocol;
use crate::recv::InfraredReceiverState;

#[cfg(test)]
mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;

/// Denon protocol
pub struct Denon;

impl InfraredProtocol for Denon {
    type Cmd = DenonCommand;
}

pub struct DenonReceiverState {
    state: DenonState,
    buf: u64,
    dt_save: u32,
    ranges: PulseWidthRange<PulseWidth>,
}

impl InfraredReceiverState for DenonReceiverState {
    fn create(samplerate: u32) -> Self {
        let ranges = PulseWidthRange::new(&nsamples());

        DenonReceiverState {
            state: DenonState::Idle,
            buf: 0,
            dt_save: 0,
            ranges,
        }
    }

    fn reset(&mut self) {
        self.state = DenonState::Idle;
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
    type InternalStatus = DenonState;

    fn event(state: &mut Self::ReceiverState, rising: bool, dt: u32) -> DenonState {
        if rising {
            let pulsewidth = state.ranges.pulsewidth(state.dt_save + dt);

            state.state = match (state.state, pulsewidth) {
                (DenonState::Idle, PulseWidth::SYNC) => DenonState::Data(0),
                (DenonState::Idle, _) => DenonState::Idle,
                (DenonState::Data(47), PulseWidth::ZERO) => DenonState::Done,
                (DenonState::Data(47), PulseWidth::ONE) => DenonState::Done,
                (DenonState::Data(idx), PulseWidth::ZERO) => DenonState::Data(idx + 1),
                (DenonState::Data(idx), PulseWidth::ONE) => {
                    state.buf |= 1 << idx;
                    DenonState::Data(idx + 1)
                }
                (DenonState::Data(_ix), _) => DenonState::Idle,
                (DenonState::Done, _) => DenonState::Done,
            };

            state.dt_save = 0;
        } else {
            state.dt_save = dt;
        }

        state.state
    }

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        if state.state == DenonState::Done {
            Some(DenonCommand { bits: state.buf })
        } else {
            None
        }
    }

}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DenonState {
    Idle,
    Data(u8),
    Done,
}

impl Into<Status> for DenonState {
    fn into(self) -> Status {
        match self {
            DenonState::Idle => Status::Idle,
            DenonState::Data(_) => Status::Receiving,
            DenonState::Done => Status::Done,
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

impl Default for PulseWidth {
    fn default() -> Self {
        PulseWidth::FAIL
    }
}

impl From<usize> for PulseWidth {
    fn from(value: usize) -> Self {
        match value {
            0 => PulseWidth::SYNC,
            1 => PulseWidth::ZERO,
            2 => PulseWidth::ONE,
            _ => PulseWidth::FAIL,
        }
    }
}

const fn nsamples() -> [(u32, u32); 4] {
    [
        // SYNC
        ((HEADER_HIGH + HEADER_LOW), 5),
        // ZERO
        ((DATA_HIGH + ZERO_LOW), 10),
        // ONE
        ((DATA_HIGH + ONE_LOW), 10),
        // Not needed. Fix when const generics arrive
        (0xFFFFFFFF, 0),
    ]
}
