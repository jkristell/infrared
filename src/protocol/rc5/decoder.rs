use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::{DecoderState, DecoderStateMachine, DecodingError, Status},
};

const RC5_BASE_TIME: u32 = 889;

impl<Time: InfraMonotonic> DecoderStateMachine<Time> for Rc5 {
    type State = Rc5ReceiverState;
    type InternalStatus = Rc5Status;
    const PULSE_LENGTHS: [u32; 8] = [RC5_BASE_TIME, 2 * RC5_BASE_TIME, 0, 0, 0, 0, 0, 0];
    const TOLERANCE: [u32; 8] = [12, 10, 0, 0, 0, 0, 0, 0];

    fn state() -> Self::State {
        Rc5ReceiverState::default()
    }

    fn new_event(
        state: &mut Self::State,
        spans: &PulseSpans<Time::Duration>,
        rising: bool,
        delta_t: Time::Duration,
    ) -> Self::InternalStatus {
        use Rc5Status::*;

        // Find this delta t in the defined ranges
        let clock_ticks = Time::find::<usize>(spans, delta_t);

        if let Some(ticks) = clock_ticks {
            state.clock += ticks + 1;
        } else {
            state.reset();
        }

        let is_odd = state.clock & 1 == 0;

        state.status = match (state.status, rising, clock_ticks) {
            (Idle, false, _) => Idle,
            (Idle, true, _) => {
                state.clock = 0;
                state.bitbuf |= 1 << 13;
                Data(12)
            }

            (Data(0), true, Some(_)) if is_odd => {
                state.bitbuf |= 1;
                Done
            }
            (Data(0), false, Some(_)) if is_odd => Done,

            (Data(bit), true, Some(_)) if is_odd => {
                state.bitbuf |= 1 << bit;
                Data(bit - 1)
            }
            (Data(bit), false, Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _, Some(_)) => Data(bit),
            (Data(_), _, None) => Err(DecodingError::Data),
            (Done, _, _) => Done,
            (Err(err), _, _) => Err(err),
        };

        state.status
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(state.bitbuf))
    }
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
