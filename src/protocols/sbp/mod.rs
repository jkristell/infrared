//! # Samsung BluRay Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used with. After the Header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use core::ops::Range;
use crate::{ReceiverStateMachine, ProtocolId, ReceiverState};
use crate::receiver::ReceiverError;


#[derive(Debug)]
pub struct SbpReceiver {
    state: SbpState,
    address: u16,
    command: u32,
    prev_sampletime: u32,
    prev_pinval: bool,
    tolerances: Tolerances,
}

#[derive(Debug)]
pub struct SbpCommand {
    pub address: u16,
    pub command: u8,
    pub valid: bool,
}

impl SbpCommand {

    pub fn from_receiver(address: u16, mut command: u32) -> Self {

        // Discard the 4 unknown bits
        command >>= 4;

        // Check the checksum
        let valid = (((command >> 0) ^ (command >> 8)) & 0xFF) == 0xFF;

        Self {
            address,
            command: (command ) as u8,
            valid
        }
    }
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum SbpState {
    // Waiting for first pulse
    Init,
    // Receiving address
    Address(u16),
    Divider,
    // Receiving data
    Command(u16),
    // Command received
    Done,
    // In error state
    Err(()),
    // Disabled
    Disabled,
}

pub type SbpResult = ReceiverState<SbpCommand>;

impl SbpReceiver {

    pub fn new(samplerate: u32) -> Self {
        Self {
            state: SbpState::Init,
            address: 0,
            command: 0,
            prev_sampletime: 0,
            prev_pinval: false,
            tolerances: Tolerances::from_timing(&DISTS, samplerate)
        }
    }

    fn receiver_state(&self) -> SbpResult {
        use ReceiverState::*;
        // Internalstate to ReceiverState
        match self.state {
            SbpState::Init => Idle,
            SbpState::Done => Done(SbpCommand::from_receiver(self.address, self.command)),
            SbpState::Err(e) => Error(ReceiverError::Data(0)), //TODO
            SbpState::Disabled => Disabled,
            _ => Receiving,
        }
    }
}


impl ReceiverStateMachine for SbpReceiver {
    type Cmd = SbpCommand;
    const PROTOCOL_ID: ProtocolId = ProtocolId::Sbp;

    fn sample(&mut self, pinval: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {

        let rising_edge = pinval && !self.prev_pinval;
        self.prev_pinval = pinval;

        if rising_edge {
            return self.sample_edge(true, sampletime);
        }

        self.receiver_state()
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {
        use SbpState::*;
        use PulseWidth::*;

        if rising {
            let mut delta = sampletime.wrapping_sub(self.prev_sampletime);

            if delta >= core::u16::MAX.into() {
                delta = 0;
            }

            self.prev_sampletime = sampletime;

            let pulsewidth = self.tolerances.pulsewidth(delta);

            let newstate = match (self.state, pulsewidth) {
                (Init,          Sync)       => Address(0),
                (Init,          _)          => Init,

                (Address(15),   One)        => {self.address |= 1 << 15; Divider},
                (Address(15),   Zero)       => Divider,
                (Address(bit),  One)        => {self.address |= 1 << bit; Address(bit + 1)},
                (Address(bit),  Zero)       => Address(bit + 1),
                (Address(_),    _)          => Err(()),

                (Divider,       Paus)       => Command(0),
                (Divider,       _)          => Err(()),

                (Command(19),   One)        => {self.command |= 1 << 19; Done},
                (Command(19),   Zero)       => Done,
                (Command(bit),  One)        => {self.command |= 1 << bit; Command(bit + 1)},
                (Command(bit),  Zero)       => Command(bit + 1),
                (Command(_),    _)          => Err(()),

                (Done,          _)          => Done,
                (Err(err),      _)          => Err(err),
                (Disabled,      _)          => Disabled,
            };

            self.state = newstate;
        }

        self.receiver_state()
    }

    fn reset(&mut self) {
        self.state = SbpState::Init;
        self.address = 0;
        self.command = 0;
        self.prev_sampletime = 0;
        self.prev_pinval = false;
    }

    fn disable(&mut self) {
        self.state = SbpState::Disabled;
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



