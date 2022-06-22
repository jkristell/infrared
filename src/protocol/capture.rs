use core::marker::PhantomData;

use crate::{
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderFactory, ProtocolDecoder, State,
    },
    Protocol,
};

impl<Mono: InfraMonotonic> DecoderFactory<Mono> for Capture<Mono> {
    type Decoder = CaptureDecoder<Mono>;
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

    fn spans(&self) -> &PulseSpans<Mono> {
        todo!()
    }
}
