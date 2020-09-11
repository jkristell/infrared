use crate::{
    protocols::rc5::Rc5Command,
    sender::{Sender, State},
};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum InternalState {
    Idle,
    // index and half of rc5 bit
    Tx(u32, bool),
    Done,
    Disabled,
}

pub struct Rc5Sender {
    pub state: InternalState,
    samples: u32,
    cmd: Rc5Command,
    pub bits: u16,
    ts: u32,
}

impl Rc5Sender {
    pub fn new(samplerate: u32) -> Self {
        let samples = (samplerate * 889) / 1_000_000;

        Self {
            state: InternalState::Disabled,
            samples,
            cmd: Rc5Command::from_bits(0),
            bits: 0,
            ts: 0,
        }
    }

    pub fn baseunits_since_last(&self, ts: u32) -> bool {
        ts.wrapping_sub(self.ts) >= self.samples
    }
}

impl Sender<Rc5Command> for Rc5Sender {
    fn load(&mut self, cmd: Rc5Command) {
        self.state = InternalState::Idle;
        self.cmd = cmd;
        self.bits = self.cmd.to_bits();
    }

    fn step(&mut self, ts: u32) -> State {
        use InternalState::*;

        let nsamples = self.baseunits_since_last(ts);

        let newstate = match (self.state, nsamples) {
            (Idle, _) => {
                // Start sending first bit, and start with the second half
                self.ts = ts;
                Tx(13, true)
            }
            (Tx(0, true), true) => Done,
            (Tx(n, false), true) => {
                self.ts = ts;
                Tx(n, true)
            }
            (Tx(n, true), true) => {
                self.ts = ts;
                Tx(n - 1, false)
            }
            (Tx(n, h), _) => Tx(n, h),
            (Done, _) => Done,
            (Disabled, _) => Disabled,
        };

        self.state = newstate;

        if let InternalState::Tx(bit, second_half) = newstate {
            let one = (self.bits & (1 << bit)) != 0;
            let pwm = (one && second_half) || (!one && !second_half);
            return State::Transmit(pwm);
        }

        State::Idle
    }

    fn reset(&mut self) {
        self.state = InternalState::Disabled;
        self.bits = 0;
        self.ts = 0;
    }
}

#[cfg(feature = "embedded-hal")]
impl crate::sender::PwmPinSender<Rc5Command> for Rc5Sender {}
