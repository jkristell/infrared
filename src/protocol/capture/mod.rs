use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    receiver::{DecoderState, DecoderStateMachine, Status},
    Protocol,
};

pub struct Capture;

pub struct CaptureState {
    pub ts: [u16; 96],
    pub pos: usize,
}

impl DecoderState for CaptureState {
    fn reset(&mut self) {
        self.ts.fill(0);
        self.pos = 0;
    }
}

impl Protocol for Capture {
    type Cmd = [u16; 96];
}

impl<Time: InfraMonotonic> DecoderStateMachine<Time> for Capture {
    type State = CaptureState;
    type InternalStatus = Status;
    const PULSE_LENGTHS: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    const TOLERANCE: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    fn state() -> Self::State {
        CaptureState {
            ts: [0; 96],
            pos: 0,
        }
    }

    fn new_event(
        state: &mut Self::State,
        _: &PulseSpans<Time::Duration>,
        _edge: bool,
        _: Time::Duration,
    ) -> Self::InternalStatus {
        if state.pos >= state.ts.len() {
            return Status::Done;
        }

        state.ts[state.pos] = 0 as u16; //TODO
        state.pos += 1;

        Status::Receiving
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(state.ts)
    }
}
