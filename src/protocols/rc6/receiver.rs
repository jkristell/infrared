use crate::recv::{Error, InfraredReceiver, State};
use crate::protocols::rc6::Rc6Command;
use crate::protocols::utils::InfraRange6;

const RC6_TIME_UNIT: u32 = 444;

const UNITS_AND_TOLERANCE: &[(u32, u32); 6] =
    &[
        (RC6_TIME_UNIT * 1, 12),
        (RC6_TIME_UNIT * 2, 12),
        (RC6_TIME_UNIT * 3, 12),
        (RC6_TIME_UNIT * 4, 12),
        (RC6_TIME_UNIT * 5, 12),
        (RC6_TIME_UNIT * 6, 12),
    ];

/// Philips Rc6
pub struct Rc6 {
    pub(crate) state: Rc6State,
    data: u32,
    headerdata: u32,
    toggle: bool,
    clock: u32,
    ranges: InfraRange6,
}

impl InfraredReceiver for Rc6 {
    type Cmd = Rc6Command;
    type InternalState = Rc6State;

    fn create() -> Self {
        Self::with_samplerate(1_000_000)
    }

    fn with_samplerate(samplerate: u32) -> Self {
        Rc6 {
            state: Rc6State::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
            ranges: InfraRange6::new(UNITS_AND_TOLERANCE, samplerate),
        }
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Rc6State {
        use Rc6State::*;

        // Find the nbr of time unit ticks the dt represents
        let ticks = self.ranges.find(dt).map(|v| (v + 1) as u32);

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
            (Data(_),       _,          None)           => Rc6Err(Error::Data),

            (Done,          _,          _)              => Done,
            (Rc6Err(err),   _,          _)              => Rc6Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(self.data, self.toggle))
    }

    fn reset(&mut self) {
        self.state = Rc6State::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.clock = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc6State {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Rc6Err(Error),
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

