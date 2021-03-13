use crate::recv::{InfraredReceiverState, InfraredReceiver, Status};
use crate::InfraredProtocol;

pub struct Capture;

pub struct CaptureState {
    pub ts: [u16; 96],
    pub pos: usize,
}

impl InfraredReceiverState for CaptureState {
    fn create(_samplerate: u32) -> Self {
        CaptureState {
            ts: [0; 96],
            pos: 0,
        }
    }

    fn reset(&mut self) {
        self.ts.iter_mut().for_each(|v| *v = 0);
        self.pos = 0;
    }
}

impl InfraredProtocol for Capture {
    type Cmd = [u16; 96];
}

impl InfraredReceiver for Capture {
    type ReceiverState = CaptureState;
    type InternalStatus = Status;

    fn event(state: &mut Self::ReceiverState, _edge: bool, dt: u32) -> Self::InternalStatus {

        if state.pos >= state.ts.len() {
            return Status::Done
        }

        state.ts[state.pos] = dt as u16;
        state.pos += 1;

        Status::Receiving
    }

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        Some(state.ts)
    }
}