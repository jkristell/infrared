use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    receiver::{DecoderData, DecoderStateMachine, State},
    Protocol,
};

pub struct Capture;

pub struct CaptureData {
    pub ts: [u16; 96],
    pub pos: usize,
}

impl DecoderData for CaptureData {
    fn reset(&mut self) {
        self.ts.fill(0);
        self.pos = 0;
    }
}

impl Protocol for Capture {
    type Cmd = [u16; 96];
}

impl<Mono: InfraMonotonic> DecoderStateMachine<Mono> for Capture {
    type Data = CaptureData;
    type InternalState = State;
    const PULSE_LENGTHS: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    const TOLERANCE: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    fn create_data() -> Self::Data {
        CaptureData {
            ts: [0; 96],
            pos: 0,
        }
    }

    fn new_event(
        state: &mut Self::Data,
        _: &PulseSpans<Mono::Duration>,
        _edge: bool,
        _: Mono::Duration,
    ) -> Self::InternalState {
        if state.pos >= state.ts.len() {
            return State::Done;
        }

        state.ts[state.pos] = 0 as u16; //TODO
        state.pos += 1;

        State::Receiving
    }

    fn command(state: &Self::Data) -> Option<Self::Cmd> {
        Some(state.ts)
    }
}
