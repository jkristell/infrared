use crate::{
    receiver::{ConstDecodeStateMachine, DecoderState, DecoderStateMachine, Status},
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

impl DecoderStateMachine for Capture {
    type State = CaptureState;
    type RangeData = ();
    type InternalStatus = Status;

    fn state() -> Self::State {
        CaptureState {
            ts: [0; 96],
            pos: 0,
        }
    }

    fn ranges(_resolution: usize) -> Self::RangeData {}

    fn event_full(
        state: &mut Self::State,
        _: &Self::RangeData,
        _edge: bool,
        dt: usize,
    ) -> Self::InternalStatus {
        if state.pos >= state.ts.len() {
            return Status::Done;
        }

        state.ts[state.pos] = dt as u16;
        state.pos += 1;

        Status::Receiving
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(state.ts)
    }
}

impl<const R: usize> ConstDecodeStateMachine<R> for Capture {
    const RANGES: Self::RangeData = ();
}
