use crate::{
    protocols::rc5::Rc5Command,
    recv::{Error, InfraredReceiver, State},
};
use crate::protocols::utils::InfraRange2;

const RC5_BASE_TIME: u32 = 889;

/// Philips Rc5
pub struct Rc5 {
    pub(crate) state: Rc5State,
    bitbuf: u16,
    pub(crate) clock: u32,
    pub(crate) ranges: InfraRange2,
}

impl InfraredReceiver for Rc5 {
    type Cmd = Rc5Command;
    type InternalState = Rc5State;

    fn create() -> Self {
        Self::with_samplerate(1_000_000)
    }

    fn with_samplerate(samplerate: u32) -> Self {
        Rc5 {
            state: Rc5State::Idle,
            bitbuf: 0,
            clock: 0,
            ranges: InfraRange2::new(&[(RC5_BASE_TIME, 10),(RC5_BASE_TIME * 2, 10)], samplerate)
        }
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> Self::InternalState {
        use Rc5State::*;

        // Find this delta t in the defined ranges
        let clock_ticks = self.ranges.find::<u32>(dt);

        if let Some(ticks) = clock_ticks {
            self.clock += ticks + 1;
        } else {
            self.reset();
        }

        let is_odd = self.clock & 1 == 0;

        self.state = match (self.state, rising, clock_ticks) {

            (Idle,      false,    _) => Idle,
            (Idle,      true,     _) => {
                self.clock = 0;
                self.bitbuf |= 1 << 13; Data(12)
            }

            (Data(0),   true,     Some(_)) if is_odd => { self.bitbuf |= 1; Done }
            (Data(0),   false,    Some(_)) if is_odd => Done,

            (Data(bit), true,     Some(_)) if is_odd => { self.bitbuf |= 1 << bit; Data(bit - 1) }
            (Data(bit), false,    Some(_)) if is_odd => Data(bit - 1),

            (Data(bit), _,          Some(_)) => Data(bit),
            (Data(_),   _,          None) => Err(Error::Data),
            (Done,      _,          _) => Done,
            (Err(err),  _,          _) => Err(err),
        };

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(Rc5Command::unpack(self.bitbuf))
    }

    fn reset(&mut self) {
        self.state = Rc5State::Idle;
        self.bitbuf = 0;
        self.clock = 0;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rc5State {
    Idle,
    Data(u8),
    Done,
    Err(Error),
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
