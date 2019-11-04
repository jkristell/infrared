use core::ops::Range;

use crate::{Receiver, ProtocolId, ReceiverState};

#[derive(Debug)]
pub struct S36Receiver {
    pub state: S36State,
    pub address: u16,
    pub bitbuf: u32,
    pub prev_sampletime: u32,
    prev_pinval: bool,
    pub tolerances: Tolerances,
    pub delta: u32,
}

#[derive(Debug)]
pub struct S36Command {
    pub address: u16,
    pub command: u8,
    pub valid: bool,
}

impl S36Command {

    pub fn from_receiver(address: u16, mut command: u32) -> Self {

        let valid = command & 7 == 7;

        command >>= 4;
        let valid = valid &&
            ((((command >> 0) ^ (command >> 8)) & 0xFF) == 0xFF);

        Self {
            address,
            command: (command ) as u8,
            valid
        }
    }
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum S36State {
    // Waiting for first pulse
    Init,
    // Receiving address
    ReceivingAddress(u32),

    Divider,

    // Receiving data
    Receiving(u32),
    // Command received
    Done,
    // In error state
    Err(()),
    // Disabled
    Disabled,
}

pub type S36Result = ReceiverState<S36Command, ()>;


impl S36Receiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            state: S36State::Init,
            delta: 0,
            address: 0,
            bitbuf: 0,
            prev_sampletime: 0,
            prev_pinval: false,
            tolerances: Tolerances::from_timing(&DISTS, samplerate)
        }
    }

    fn receiver_state(&self) -> S36Result {
        use ReceiverState::*;
        // Internalstate to ReceiverState
        match self.state {
            S36State::Init => Idle,
            S36State::Done => Done(S36Command::from_receiver(self.address, self.bitbuf)),
            S36State::Err(e) => Error(e),
            S36State::Disabled => Disabled,
            _ => Receiving,
        }
    }

}


impl Receiver for S36Receiver {
    type Cmd = S36Command;
    type Err = ();
    const PROTOCOL_ID: ProtocolId = ProtocolId::S36;

    fn sample(&mut self, pinval: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err> {

        let rising_edge = pinval && !self.prev_pinval;
        self.prev_pinval = pinval;

        if rising_edge {
            return self.sample_edge(true, sampletime);
        }

        self.receiver_state()

    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err> {
        use S36State::*;
        use PulseWidth::*;

        if rising {
            let mut delta = sampletime.wrapping_sub(self.prev_sampletime);

            if delta >= core::u16::MAX.into() {
                delta = 0;
            }

            self.prev_sampletime = sampletime;
            self.delta = delta;

            let pulsewidth = self.tolerances.pulsewidth(delta);

            let newstate = match (self.state, pulsewidth) {
                (Init,            Sync)     => ReceivingAddress(0),
                (Init,            _)        => Init,

                (ReceivingAddress(15),   One)      => {self.address |= 1 << 15; Divider},
                (ReceivingAddress(15),   Zero)     => Divider,
                (ReceivingAddress(bit),  One)      => {self.address |= 1 << bit; ReceivingAddress(bit + 1)},
                (ReceivingAddress(bit),  Zero)     => ReceivingAddress(bit + 1),
                (ReceivingAddress(_),    _)        => Err(()),

                (Divider, Paus)             => Receiving(0),
                (Divider, _) => Err(()),

                (Receiving(19),   One)      => {self.bitbuf |= 1 << 19; Done},
                (Receiving(19),   Zero)     => Done,

                (Receiving(bit),  One)      => {self.bitbuf |= 1 << bit; Receiving(bit + 1)},
                (Receiving(bit),  Zero)     => Receiving(bit + 1),

                (Receiving(_),    _)        => Err(()),

                (Done,            _)        => Done,
                (Err(err),        _)        => Err(err),
                (Disabled,        _)        => Disabled,
            };

            self.state = newstate;
        }

        self.receiver_state()
    }

    fn reset(&mut self) {
        self.state = S36State::Init;
        self.address = 0;
        self.bitbuf = 0;
        self.prev_sampletime = 0;
        self.prev_pinval = false;
    }

    fn disable(&mut self) {
        self.state = S36State::Disabled;
    }
}

#[derive(Debug, Clone)]
pub struct Tolerances {
    pub sync: Range<u32>,
    pub paus: Range<u32>,
    pub zero: Range<u32>,
    pub one: Range<u32>,
}

pub enum PulseWidth {
    Sync,
    Paus,
    Zero,
    One,
    NotAPulseWidth,
}

impl Tolerances {
    pub const fn from_timing(timing: &Pulsedistance, samplerate: u32) -> Self {
        let per: u32 = 1000 / (samplerate / 1000);
        Tolerances {
            sync: sample_range((timing.header_high + timing.header_low) / per, 5),
            paus: sample_range((timing.data_high + timing.paus) / per, 5),
            zero: sample_range((timing.data_high + timing.zero_low) / per, 10),
            one: sample_range((timing.data_high + timing.one_low) / per, 10),
        }
    }

    pub fn pulsewidth(&self, samples: u32) -> PulseWidth {
        if self.sync.contains(&samples) {
            return PulseWidth::Sync;
        }
        if self.paus.contains(&samples) {
            return PulseWidth::Paus;
        }
        if self.one.contains(&samples) {
            return PulseWidth::One;
        }
        if self.zero.contains(&samples) {
            return PulseWidth::Zero;
        }
        PulseWidth::NotAPulseWidth
    }
}

const fn sample_range(units: u32, percent: u32) -> Range<u32> {
    let tol = (units * percent) / 100;
    (units - tol..units + tol)
}
pub struct Pulsedistance {
    header_high: u32,
    header_low: u32,

    paus: u32,

    data_high: u32,
    zero_low: u32,
    one_low: u32,
}

const DISTS: Pulsedistance = Pulsedistance {
    header_high: 4500,
    header_low: 4500,
    paus: 4500,
    zero_low: 500,
    data_high: 500,
    one_low: 1500,
};



