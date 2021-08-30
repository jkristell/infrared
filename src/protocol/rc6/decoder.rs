use crate::{
    protocol::{rc6::Rc6Command, utils::InfraConstRange, Rc6},
    receiver::{ConstDecodeStateMachine, DecoderState, DecoderStateMachine, DecodingError, Status},
};

const RC6_TIME_UNIT: usize = 444;

const TOLERANCE: usize = 12;

const UNITS_AND_TOLERANCE: &[(usize, usize); 6] = &[
    (RC6_TIME_UNIT * 1, TOLERANCE),
    (RC6_TIME_UNIT * 2, TOLERANCE),
    (RC6_TIME_UNIT * 3, TOLERANCE),
    (RC6_TIME_UNIT * 4, TOLERANCE),
    (RC6_TIME_UNIT * 5, TOLERANCE),
    (RC6_TIME_UNIT * 6, TOLERANCE),
];

impl DecoderStateMachine for Rc6 {
    type State = Rc6ReceiverState;
    type RangeData = InfraConstRange<6>;
    type InternalStatus = Rc6Status;

    fn state() -> Self::State {
        Self::State {
            state: Rc6Status::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
        }
    }

    fn ranges(resolution: usize) -> Self::RangeData {
        InfraConstRange::new(UNITS_AND_TOLERANCE, resolution)
    }

    #[rustfmt::skip]
    fn event_full(state: &mut Rc6ReceiverState, ranges: &Self::RangeData, rising: bool, dt: usize) -> Rc6Status {
        use Rc6Status::*;

        // Find the nbr of time unit ticks the dt represents
        let ticks = ranges.find::<usize>(dt).map(|v| (v + 1));

        // Reconstruct the clock
        if let Some(ticks) = ticks {
            state.clock += ticks;
        } else {
            state.reset();
        }

        let odd = state.clock & 1 == 1;

        state.state = match (state.state, rising, ticks) {
            (Idle,          false,    _)            => Idle,
            (Idle,          true,     _)            => { state.clock = 0; Leading },
            (Leading,       false,    Some(6))      => LeadingPaus,
            (Leading,       _,        _)            => Idle,
            (LeadingPaus,   true,     Some(2))      => HeaderData(3),
            (LeadingPaus,   _,        _)            => Idle,

            (HeaderData(n), _,          Some(_)) if odd => {
                state.headerdata |= if rising { 0 } else { 1 } << n;
                if n == 0 {
                    Trailing
                } else {
                    HeaderData(n - 1)
                }
            }

            (HeaderData(n), _,          Some(_))    => HeaderData(n),
            (HeaderData(_), _,          None)       => Idle,

            (Trailing,      false,      Some(3))    => { state.toggle = true; Data(15) }
            (Trailing,      true,       Some(2))    => { state.toggle = false; Data(15) }
            (Trailing,      false,      Some(1))    => Trailing,
            (Trailing,      _,          _)          => Idle,

            (Data(0),       true,       Some(_)) if odd => Done,
            (Data(0),       false,      Some(_)) if odd => { state.data |= 1; Done }
            (Data(0),       _,          Some(_))        => Data(0),
            (Data(n),       true,       Some(_)) if odd => Data(n - 1),
            (Data(n),       false,      Some(_)) if odd => { state.data |= 1 << n; Data(n - 1) }
            (Data(n),       _,          Some(_))        => Data(n),
            (Data(_),       _,          None)           => Rc6Err(DecodingError::Data),

            (Done,          _,          _)              => Done,
            (Rc6Err(err),   _,          _)              => Rc6Err(err),
        };

        state.state
    }

    fn command(state: &Rc6ReceiverState) -> Option<Self::Cmd> {
        Some(Rc6Command::from_bits(state.data, state.toggle))
    }
}

impl<const R: usize> ConstDecodeStateMachine<R> for Rc6 {
    const RANGES: Self::RangeData = InfraConstRange::new(UNITS_AND_TOLERANCE, R);
}

pub struct Rc6ReceiverState {
    pub(crate) state: Rc6Status,
    data: u16,
    headerdata: u16,
    toggle: bool,
    clock: usize,
}

impl DecoderState for Rc6ReceiverState {
    fn reset(&mut self) {
        self.state = Rc6Status::Idle;
        self.data = 0;
        self.headerdata = 0;
        self.clock = 0;
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rc6Status {
    Idle,
    Leading,
    LeadingPaus,
    HeaderData(u32),
    Trailing,
    Data(u32),
    Done,
    Rc6Err(DecodingError),
}

impl From<Rc6Status> for Status {
    fn from(state: Rc6Status) -> Self {
        match state {
            Rc6Status::Idle => Status::Idle,
            Rc6Status::Done => Status::Done,
            Rc6Status::Rc6Err(err) => Status::Error(err),
            _ => Status::Receiving,
        }
    }
}
