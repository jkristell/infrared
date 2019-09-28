
use crate::prelude::*;
use crate::rc5::Rc5Command;

enum State {
    Idle,
    Tx,
    Done,
}


pub struct Rc5Transmitter {
    cmd: Rc5Command,
    bits: u16,
    bitidx: u32,
    ts: u32,
}

impl Rc5Transmitter {
    pub fn new() -> Self {
        Self {
            cmd: Rc5Command::from_bits(0),
            bits: 0,
            bitidx: 0,
            ts: 0
        }
    }
}


impl Transmitter<Rc5Command> for Rc5Transmitter {
    fn load(&mut self, cmd: Rc5Command) {
        self.cmd = cmd;
    }

    fn step(&mut self, ts: u32) -> TransmitterState {




        TransmitterState::Transmit(false)
    }

    fn reset(&mut self) {
        unimplemented!()
    }
}