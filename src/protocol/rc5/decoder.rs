use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::{DecoderData, Decoder, DecodingError, State},
};

const RC5_BASE_TIME: u32 = 889;

impl<Mono: InfraMonotonic> Decoder<Mono> for Rc5 {
    type Data = Rc5Data<Mono>;
    type InternalState = Rc5State;
    const PULSE: [u32; 8] = [RC5_BASE_TIME, 2 * RC5_BASE_TIME, 0, 0, 0, 0, 0, 0];
    const TOL: [u32; 8] = [12, 10, 0, 0, 0, 0, 0, 0];

    fn decoder(freq: u32) -> Self::Data {
        Rc5Data {
            state: Rc5State::Idle,
            bitbuf: 0,
            clock: 0,
            spans: <Self as Decoder<Mono>>::create_pulsespans(freq),
        }
    }

    fn event(
        self_: &mut Self::Data,
        rising: bool,
        delta_t: Mono::Duration,
    ) -> Self::InternalState {
        use Rc5State::*;

        // Find this delta t in the defined ranges
        let clock_ticks = Mono::find::<usize>(&self_.spans, delta_t);

        if let Some(ticks) = clock_ticks {
            self_.clock += ticks + 1;
        } else {
            self_.reset();
        }

        let is_odd = self_.clock & 1 == 0;

        self_.state = match (self_.state, rising, clock_ticks) {
            (Idle, false, _) => Idle,
            (Idle, true, _) => {
                self_.clock = 0;
                self_.bitbuf |= 1 << 13;
                Data(12)
            }

            (Data(0), true, Some(_)) if is_odd => {
                self_.bitbuf |= 1;
                Done
            }
            (Data(0), false, Some(_)) if is_odd => Done,

            (Data(bit), true, Some(_)) if is_odd => {
                self_.bitbuf |= 1 << bit;
                Data(bit - 1)
            }
            (Data(bit), false, Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _, Some(_)) => Data(bit),
            (Data(_), _, None) => Err(DecodingError::Data),
            (Done, _, _) => Done,
            (Err(err), _, _) => Err(err),
        };

        self_.state
    }

    fn command(state: &Self::Data) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(state.bitbuf))
    }
}

pub struct Rc5Data<Mono: InfraMonotonic> {
    pub(crate) state: Rc5State,
    bitbuf: u16,
    pub(crate) clock: usize,
    spans: PulseSpans<Mono::Duration>,
}

impl<Mono: InfraMonotonic> DecoderData for Rc5Data<Mono> {
    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }
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
