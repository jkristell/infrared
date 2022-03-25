use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{rc6::Rc6Command, Rc6},
    receiver::{DecoderData, DecoderStateMachine, DecodingError, State},
};

const RC6_TIME_UNIT: u32 = 444;

impl<Mono: InfraMonotonic> DecoderStateMachine<Mono> for Rc6 {
    type Data = Rc6Data;
    type InternalState = Rc6State;
    const PULSE_LENGTHS: [u32; 8] = [
        RC6_TIME_UNIT,
        RC6_TIME_UNIT * 2,
        RC6_TIME_UNIT * 3,
        RC6_TIME_UNIT * 4,
        RC6_TIME_UNIT * 5,
        RC6_TIME_UNIT * 6,
        0,
        0,
    ];
    const TOLERANCE: [u32; 8] = [12, 12, 12, 12, 12, 12, 12, 12];

    fn create_data() -> Self::Data {
        Self::Data {
            state: Rc6State::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
        }
    }

    #[rustfmt::skip]
    fn event(data: &mut Self::Data, spans: &PulseSpans<Mono::Duration>, rising: bool, dt: Mono::Duration)
             -> Self::InternalState {
        use Rc6State::*;

        // Find the nbr of time unit ticks the dt represents
        //let ticks = ranges.find::<usize>(dt).map(|v| (v + 1));
        let ticks = Mono::find::<usize>(spans, dt).map(|v| (v +1) );

        // Reconstruct the clock
        if let Some(ticks) = ticks {
            data.clock += ticks;
        } else {
            data.reset();
        }

        let odd = data.clock & 1 == 1;

        data.state = match (data.state, rising, ticks) {
            (Idle,          false,    _)            => Idle,
            (Idle,          true,     _)            => { data.clock = 0; Leading },
            (Leading,       false,    Some(6))      => LeadingPaus,
            (Leading,       _,        _)            => Idle,
            (LeadingPaus,   true,     Some(2))      => HeaderData(3),
            (LeadingPaus,   _,        _)            => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                data.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      false,      Some(3))    => { data.toggle = true; Data(15) }
            (Trailing,      true,       Some(2))    => { data.toggle = false; Data(15) }
            (Trailing,      false,      Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       true,       Some(_)) if odd => Done,
            (Data(0),       false,      Some(_)) if odd => { data.data |= 1; Done }
            (Data(0),       _,          Some(_))        => Data(0),
            (Data(n),       true,       Some(_)) if odd => Data(n - 1),
            (Data(n),       false,      Some(_)) if odd => { data.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))        => Data(n),
            (Data(_),       _,          None)           => Rc6Err(DecodingError::Data),

            (Done,          _,          _)              => Done,
            (Rc6Err(err),   _,          _)              => Rc6Err(err),
        };

        data.state

    }

    fn command(data: &Rc6Data) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(data.data, data.toggle))
    }
}


pub struct Rc6Data {
    pub(crate) state: Rc6State,
    data: u16,
    headerdata: u16,
    toggle: bool,
    clock: usize,
}

impl DecoderData for Rc6Data {
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
