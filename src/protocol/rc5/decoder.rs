use crate::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderFactory, DecodingError, ProtocolDecoder, State,
    },
};

const RC5_BASE_TIME: u32 = 889;
const PULSE: [u32; 8] = [RC5_BASE_TIME, 2 * RC5_BASE_TIME, 0, 0, 0, 0, 0, 0];
const TOL: [u32; 8] = [12, 10, 0, 0, 0, 0, 0, 0];

impl<Mono: InfraMonotonic> DecoderFactory<Mono> for Rc5 {
    type Decoder = Rc5Decoder<Mono>;

    fn decoder(freq: u32) -> Self::Decoder {
        Rc5Decoder {
            state: Rc5State::Idle,
            bitbuf: 0,
            clock: 0,
            spans: PulseSpans::new(freq, &PULSE, &TOL),
        }
    }
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Mono, Rc5Command> for Rc5Decoder<Mono> {
    fn event(&mut self, rising: bool, delta_t: Mono::Duration) -> State {
        use Rc5State::*;

        // Find this delta t in the defined ranges
        let clock_ticks = self.spans.get::<usize>(delta_t);

        if let Some(ticks) = clock_ticks {
            self.clock += ticks + 1;
        } else {
            self.reset();
        }

        let is_odd = self.clock & 1 == 0;

        self.state = match (self.state, rising, clock_ticks) {
            (Idle, false, _) => Idle,
            (Idle, true, _) => {
                self.clock = 0;
                self.bitbuf |= 1 << 13;
                Data(12)
            }

            (Data(0), true, Some(_)) if is_odd => {
                self.bitbuf |= 1;
                Done
            }
            (Data(0), false, Some(_)) if is_odd => Done,

            (Data(bit), true, Some(_)) if is_odd => {
                self.bitbuf |= 1 << bit;
                Data(bit - 1)
            }
            (Data(bit), false, Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _, Some(_)) => Data(bit),
            (Data(_), _, None) => Err(DecodingError::Data),
            (Done, _, _) => Done,
            (Err(err), _, _) => Err(err),
        };

        self.state.into()
    }

    fn command(&self) -> Option<Rc5Command> {
        Some(Rc5Command::unpack(self.bitbuf))
    }

    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.spans
    }
}

pub struct Rc5Decoder<Mono: InfraMonotonic> {
    pub(crate) state: Rc5State,
    bitbuf: u16,
    pub(crate) clock: usize,
    spans: PulseSpans<Mono>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rc5State {
    Idle,
    Data(u8),
    Done,
    Err(DecodingError),
}

impl Default for Rc5State {
    fn default() -> Self {
        Rc5State::Idle
    }
}

impl From<Rc5State> for State {
    fn from(rs: Rc5State) -> Self {
        match rs {
            Rc5State::Idle => State::Idle,
            Rc5State::Data(_) => State::Receiving,
            Rc5State::Done => State::Done,
            Rc5State::Err(e) => State::Error(e),
        }
    }
}
