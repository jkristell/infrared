use crate::{
    protocol::{rc6::Rc6Command, Rc6},
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecodingError, ProtocolDecoder, DecoderAdapter, State,
    },
};

const RC6_TIME_UNIT: u32 = 444;

impl<Mono: InfraMonotonic> DecoderAdapter<Mono> for Rc6 {
    type Decoder = Rc6Decoder<Mono>;
    const PULSE: [u32; 8] = [
        RC6_TIME_UNIT,
        RC6_TIME_UNIT * 2,
        RC6_TIME_UNIT * 3,
        RC6_TIME_UNIT * 4,
        RC6_TIME_UNIT * 5,
        RC6_TIME_UNIT * 6,
        0,
        0,
    ];
    const TOL: [u32; 8] = [12, 12, 12, 12, 12, 12, 12, 12];

    fn decoder(freq: u32) -> Self::Decoder {
        Rc6Decoder {
            state: Rc6State::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
            spans: <Self as DecoderAdapter<Mono>>::create_pulsespans(freq),
        }
    }
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Mono, Rc6Command> for Rc6Decoder<Mono> {
    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: Mono::Duration) -> State {
        use Rc6State::*;

        // Find the nbr of time unit ticks the dt represents
        let ticks = self.spans.get::<usize>(dt).map(|v| v + 1);

        // Reconstruct the clock
        if let Some(ticks) = ticks {
            self.clock += ticks;
        } else {
            self.reset();
        }

        let odd = self.clock & 1 == 1;

        self.state = match (self.state, rising, ticks) {
            (Idle,          false,    _)            => Idle,
            (Idle,          true,     _)            => { self.clock = 0; Leading },
            (Leading,       false,    Some(6))      => LeadingPaus,
            (Leading,       _,        _)            => Idle,
            (LeadingPaus,   true,     Some(2))      => HeaderData(3),
            (LeadingPaus,   _,        _)            => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                self.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      false,      Some(3))    => { self.toggle = true; Data(15) }
            (Trailing,      true,       Some(2))    => { self.toggle = false; Data(15) }
            (Trailing,      false,      Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       true,       Some(_)) if odd => Done,
            (Data(0),       false,      Some(_)) if odd => { self.data |= 1; Done }
            (Data(0),       _,          Some(_))        => Data(0),
            (Data(n),       true,       Some(_)) if odd => Data(n - 1),
            (Data(n),       false,      Some(_)) if odd => { self.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))        => Data(n),
            (Data(_),       _,          None)           => Rc6Err(DecodingError::Data),

            (Done,          _,          _)              => Done,
            (Rc6Err(err),   _,          _)              => Rc6Err(err),
        };

        self.state.into()

    }

    fn command(&self) -> Option<Rc6Command> {
        Some(Rc6Command::from_bits(self.data, self.toggle))
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.clock = 0;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.spans
    }
}

pub struct Rc6Decoder<Mono: InfraMonotonic> {
    pub(crate) state: Rc6State,
    data: u16,
    headerdata: u16,
    toggle: bool,
    clock: usize,
    spans: PulseSpans<Mono>,
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Rc6Err(DecodingError),
}

impl From<Rc6State> for State {
    fn from(state: Rc6State) -> Self {
        match state {
            Rc6State::Idle => State::Idle,
            Rc6State::Done => State::Done,
            Rc6State::Rc6Err(err) => State::Error(err),
            _ => State::Receiving,
        }
    }
}
