use crate::{
    protocol::{rc5::Rc5Command, utils::InfraConstRange, Rc5},
    receiver::{ConstDecodeStateMachine, DecoderState, DecoderStateMachine, DecodingError, Status},
};

const RC5_BASE_TIME: u32 = 889;

impl DecoderStateMachine for Rc5 {
    type InternalStatus = Rc5Status;
    type State = Rc5ReceiverState;
    type RangeData = InfraConstRange<2>;

    fn state() -> Self::State {
        Rc5ReceiverState::default()
    }

    fn ranges(resolution: u32) -> Self::RangeData {
        InfraConstRange::new(&[(RC5_BASE_TIME, 12), (RC5_BASE_TIME * 2, 10)], resolution)
    }

    #[rustfmt::skip]
    fn event_full(state: &mut Self::State, ranges: &Self::RangeData, rising: bool, dt: u32) -> Self::InternalStatus {
        use Rc5Status::*;

        // Find this delta t in the defined ranges
        let clock_ticks = ranges.find::<usize>(dt);

        if let Some(ticks) = clock_ticks {
            state.clock += ticks + 1;
        } else {
            state.reset();
        }

        let is_odd = state.clock & 1 == 0;

        state.status = match (state.status, rising, clock_ticks) {

            (Idle,      false,    _) => Idle,
            (Idle,      true,     _) => {
                state.clock = 0;
                state.bitbuf |= 1 << 13; Data(12)
            }

            (Data(0),   true,     Some(_)) if is_odd => { state.bitbuf |= 1; Done }
            (Data(0),   false,    Some(_)) if is_odd => Done,

            (Data(bit), true,     Some(_)) if is_odd => { state.bitbuf |= 1 << bit; Data(bit - 1) }
            (Data(bit), false,    Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _,          Some(_)) => Data(bit),
            (Data(_),   _,          None) => Err(DecodingError::Data),
            (Done,      _,          _) => Done,
            (Err(err),  _,          _) => Err(err),
        };

        state.status
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(state.bitbuf))
    }
}

impl<const R: u32> ConstDecodeStateMachine<R> for Rc5 {
    const RANGES: Self::RangeData =
        InfraConstRange::new(&[(RC5_BASE_TIME, 10), (RC5_BASE_TIME * 2, 10)], R);
}

#[derive(Default)]
pub struct Rc5ReceiverState {
    pub(crate) status: Rc5Status,
    bitbuf: u16,
    pub(crate) clock: usize,
}

impl DecoderState for Rc5ReceiverState {
    fn reset(&mut self) {
        self.status = Rc5Status::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Rc5Status {
    Idle,
    Data(u8),
    Done,
    Err(DecodingError),
}

impl Default for Rc5Status {
    fn default() -> Self {
        Rc5Status::Idle
    }
}

impl From<Rc5Status> for Status {
    fn from(rs: Rc5Status) -> Self {
        match rs {
            Rc5Status::Idle => Status::Idle,
            Rc5Status::Data(_) => Status::Receiving,
            Rc5Status::Done => Status::Done,
            Rc5Status::Err(e) => Status::Error(e),
        }
    }
}
