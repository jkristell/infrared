use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{rc6::Rc6Command, Rc6},
    receiver::{DecoderState, DecoderStateMachine, DecodingError, Status},
};

const RC6_TIME_UNIT: u32 = 444;

impl<Time: InfraMonotonic> DecoderStateMachine<Time> for Rc6 {
    type State = Rc6ReceiverState;
    type InternalStatus = Rc6Status;
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

    fn state() -> Self::State {
        Self::State {
            state: Rc6Status::Idle,
            data: 0,
            headerdata: 0,
            toggle: false,
            clock: 0,
        }
    }

    #[rustfmt::skip]
    fn new_event(state: &mut Self::State, spans: &PulseSpans<Time::Duration>, rising: bool, dt: Time::Duration)
                 -> Self::InternalStatus {
        use Rc6Status::*;

        // Find the nbr of time unit ticks the dt represents
        //let ticks = ranges.find::<usize>(dt).map(|v| (v + 1));
        let ticks = Time::find::<usize>(spans, dt).map(|v| (v +1) );

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

/*
impl<const R: u32> ConstDecodeStateMachine<R> for Rc6 {
    const RANGES: Self::RangeData = InfraConstRange::new(UNITS_AND_TOLERANCE, R);
}

 */

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
