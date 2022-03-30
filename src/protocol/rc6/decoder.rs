use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{rc6::Rc6Command, Rc6},
    receiver::{DecoderData, Decoder, DecodingError, State},
};

const RC6_TIME_UNIT: u32 = 444;

impl<Mono: InfraMonotonic> Decoder<Mono> for Rc6 {
    type Data = Rc6Data<Mono>;
    type InternalState = Rc6State;
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

    fn decoder(freq: u32) -> Self::Data {
        Self::Data {
            state: Rc6State::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
            spans: <Self as Decoder<Mono>>::create_pulsespans(freq),
        }
    }

    #[rustfmt::skip]
    fn event(this_: &mut Self::Data, rising: bool, dt: Mono::Duration) -> Self::InternalState {
        use Rc6State::*;

        // Find the nbr of time unit ticks the dt represents
        //let ticks = ranges.find::<usize>(dt).map(|v| (v + 1));
        let ticks = Mono::find::<usize>(&this_.spans, dt).map(|v| (v +1) );

        // Reconstruct the clock
        if let Some(ticks) = ticks {
            this_.clock += ticks;
        } else {
            this_.reset();
        }

        let odd = this_.clock & 1 == 1;

        this_.state = match (this_.state, rising, ticks) {
            (Idle,          false,    _)            => Idle,
            (Idle,          true,     _)            => { this_.clock = 0; Leading },
            (Leading,       false,    Some(6))      => LeadingPaus,
            (Leading,       _,        _)            => Idle,
            (LeadingPaus,   true,     Some(2))      => HeaderData(3),
            (LeadingPaus,   _,        _)            => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                this_.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      false,      Some(3))    => { this_.toggle = true; Data(15) }
            (Trailing,      true,       Some(2))    => { this_.toggle = false; Data(15) }
            (Trailing,      false,      Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       true,       Some(_)) if odd => Done,
            (Data(0),       false,      Some(_)) if odd => { this_.data |= 1; Done }
            (Data(0),       _,          Some(_))        => Data(0),
            (Data(n),       true,       Some(_)) if odd => Data(n - 1),
            (Data(n),       false,      Some(_)) if odd => { this_.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))        => Data(n),
            (Data(_),       _,          None)           => Rc6Err(DecodingError::Data),

            (Done,          _,          _)              => Done,
            (Rc6Err(err),   _,          _)              => Rc6Err(err),
        };

        this_.state

    }

    fn command(this_: &Self::Data) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(this_.data, this_.toggle))
    }
}

pub struct Rc6Data<Mono: InfraMonotonic> {
    pub(crate) state: Rc6State,
    data: u16,
    headerdata: u16,
    toggle: bool,
    clock: usize,
    spans: PulseSpans<Mono::Duration>,
}

impl<Mono: InfraMonotonic> DecoderData for Rc6Data<Mono> {
    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.clock = 0;
    }
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
