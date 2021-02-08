use crate::protocols::utils::InfraRange2;
use crate::protocols::Rc5;
use crate::recv::InfraredReceiverState;
use crate::{
    protocols::rc5::Rc5Command,
    recv::{Error, InfraredReceiver, Status},
};

const RC5_BASE_TIME: u32 = 889;

pub struct Rc5ReceiverState {
    pub(crate) state: Rc5State,
    bitbuf: u16,
    pub(crate) clock: u32,
    pub(crate) ranges: InfraRange2,
}

impl InfraredReceiverState for Rc5ReceiverState {
    fn create(samplerate: u32) -> Self {
        Rc5ReceiverState {
            state: Rc5State::Idle,
            bitbuf: 0,
            clock: 0,
            ranges: InfraRange2::new(&[(RC5_BASE_TIME, 10), (RC5_BASE_TIME * 2, 10)], samplerate),
        }
    }

    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }
}

impl InfraredReceiver for Rc5 {
    type ReceiverState = Rc5ReceiverState;
    type InternalStatus = Rc5State;

    #[rustfmt::skip]
    fn event(state :&mut Self::ReceiverState, rising: bool, dt: u32) -> Self::InternalStatus {
        use Rc5State::*;

        // Find this delta t in the defined ranges
        let clock_ticks = state.ranges.find::<u32>(dt);

        if let Some(ticks) = clock_ticks {
            state.clock += ticks + 1;
        } else {
            state.reset(
            );
        }

        let is_odd = state.clock & 1 == 0;

        state.state = match (state.state, rising, clock_ticks) {

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
            (Data(_),   _,          None) => Err(Error::Data),
            (Done,      _,          _) => Done,
            (Err(err),  _,          _) => Err(err),
        };

        state.state
    }

    fn command(state: &Self::ReceiverState) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(state.bitbuf))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5State {
    Idle,
    Data(u8),
    Done,
    Err(Error),
}

impl From<Rc5State> for Status {
    fn from(rs: Rc5State) -> Self {
        match rs {
            Rc5State::Idle => Status::Idle,
            Rc5State::Data(_) => Status::Receiving,
            Rc5State::Done => Status::Done,
            Rc5State::Err(e) => Status::Error(e),
        }
    }
}
