use core::marker::PhantomData;

use crate::{
    receiver::{time::InfraMonotonic, ProtocolDecoder, ProtocolDecoderAdaptor, State},
    Protocol,
};

impl<Mono: InfraMonotonic> ProtocolDecoderAdaptor<Mono> for Capture<Mono> {
    type Decoder = CaptureDecoder<Mono>;

    const PULSE: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    const TOL: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

    fn decoder(_freq: u32) -> Self::Decoder {
        CaptureDecoder {
            ts: [Mono::ZERO_DURATION; 96],
            pos: 0,
        }
    }
}

pub struct Capture<Mono> {
    dur: PhantomData<Mono>,
}

pub struct CaptureDecoder<Mono: InfraMonotonic> {
    pub ts: [Mono::Duration; 96],
    pub pos: usize,
}

impl<Mono: InfraMonotonic> Protocol for Capture<Mono> {
    type Cmd = [Mono::Duration; 96];
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Mono, [Mono::Duration; 96]> for CaptureDecoder<Mono> {
    //type Cmd = [Mono::Duration; 96];
    //type InternalState = State;
    fn event(&mut self, _edge: bool, dur: Mono::Duration) -> State {
        if self.pos >= self.ts.len() {
            return State::Done;
        }

        self.ts[self.pos] = dur;
        self.pos += 1;

        State::Receiving
    }

    fn command(&self) -> Option<[Mono::Duration; 96]> {
        Some(self.ts)
    }

    fn reset(&mut self) {
        self.pos = 0;
    }
}
