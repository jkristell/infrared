use crate::{
    Receiver, ReceiverState,
};

// Timing (us)
const HEADER_LEADING: u32 = 2666;
const HEADER_TRAILING: u32 = 1350;
const DATA: u32 = 444;


pub struct PhilipsReceiver {
    samplerate: u32,
    state: InternalState,
    last: u32,
    pinval: bool,
}

enum DataType {
    Leading,
    Trailing,
    DataZero,
    DataOne,
}

impl PhilipsReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            samplerate,
            last: 0,
            state: InternalState::Idle,
            pinval: false,
        }
    }

    fn match_delta(&self, delta: u32) -> DataType {



        DataType::Leading
    }
}



enum InternalState {
    Idle,
    Leading,
    Header,
    Trailing,
    Data,
    Done,
}

impl Receiver for PhilipsReceiver {
    type Command = ();
    type ReceiveError = ();

    fn event(&mut self, rising: bool, timestamp: u32) -> ReceiverState<Self::Command, Self::ReceiveError> {

        if self.pinval != rising {

            let intern = match (self.state, rising) {
                (InternalState::Idle, false) => {
                    //TODO: Unreachable
                    InternalState::Idle
                },
                (InternalState::Idle, true) => {
                    self.last = timestamp;
                    InternalState::Leading
                },
                (InternalState::Leading, false) => {
                    // Leading header
                    let delta = timestamp - self.last;
                    //self.is_leader(delta)
                    InternalState::Header
                },
                (InternalState::Leading, _) => InternalState::Header,
                (InternalState::Header, _) => InternalState::Trailing,
                (InternalState::Trailing, _) => InternalState::Data,
                (InternalState::Data, _) => InternalState::Done,
                (InternalState::Done, _) => InternalState::Done,
            };

            self.state = intern;
        }

        match self.state {
            _ => ReceiverState::Receiving
        }
    }

    fn reset(&mut self) {
        unimplemented!()
    }

    fn disable(&mut self) {
        unimplemented!()
    }
}