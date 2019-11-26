//! # Samsung BluRay Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used with. After the Header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use crate::prelude::*;
use crate::protocols::utils::Ranges;
use crate::receiver::ReceiverError;

#[derive(Debug)]
pub struct Sbp {
    state: SbpState,
    address: u16,
    command: u32,
    prev_sampletime: u32,
    prev_pinval: bool,
    ranges: Ranges<SbpPulse>,
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
            command: (command) as u8,
            valid,
        }
    }
}

impl Command for SbpCommand {
    fn construct(address: u16, command: u8) -> Self {
        SbpCommand {
            address,
            command,
            valid: true,
        }
    }

    fn address(&self) -> u16 {
        self.address
    }

    fn command(&self) -> u8 {
        self.command
    }
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum SbpState {
    // Waiting for first pulse
    Init,
    // Receiving address
    Address(u16),
    /// Paus
    Divider,
    // Receiving data
    Command(u16),
    // Command received
    Done,
    // In error state
    Err(ReceiverError),
    // Disabled
    Disabled,
}

pub type SbpResult = ReceiverState<SbpCommand>;

impl Sbp {
    pub fn new(samplerate: u32) -> Self {
        let nsamples = nsamples_from_timing(&TIMING, samplerate);
        let ranges = Ranges::new(&nsamples);

        Self {
            state: SbpState::Init,
            address: 0,
            command: 0,
            prev_sampletime: 0,
            prev_pinval: false,
            ranges,
        }
    }

    fn receiver_state(&self) -> SbpResult {
        use SbpState::*;
        // Internalstate to ReceiverState
        match self.state {
            Init => ReceiverState::Idle,
            Done => ReceiverState::Done(SbpCommand::from_receiver(self.address, self.command)),
            Err(e) => ReceiverState::Error(e),
            Disabled => ReceiverState::Disabled,
            _ => ReceiverState::Receiving,
        }
    }
}

impl ReceiverStateMachine for Sbp {
    type Cmd = SbpCommand;
    const ID: ProtocolId = ProtocolId::Sbp;

    fn for_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate)
    }

    fn event(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {
        use SbpPulse::*;
        use SbpState::*;

        if rising {
            let mut delta = sampletime.wrapping_sub(self.prev_sampletime);

            if delta >= core::u16::MAX.into() {
                delta = 0;
            }

            self.prev_sampletime = sampletime;

            let pulsewidth = self.ranges.pulsewidth(delta);

            let newstate = match (self.state, pulsewidth) {
                (Init, Sync) => Address(0),
                (Init, _) => Init,

                (Address(15), One) => {
                    self.address |= 1 << 15;
                    Divider
                }
                (Address(15), Zero) => Divider,
                (Address(bit), One) => {
                    self.address |= 1 << bit;
                    Address(bit + 1)
                }
                (Address(bit), Zero) => Address(bit + 1),
                (Address(_), _) => Err(ReceiverError::Address(0)),

                (Divider, Paus) => Command(0),
                (Divider, _) => Err(ReceiverError::Data(0)),

                (Command(19), One) => {
                    self.command |= 1 << 19;
                    Done
                }
                (Command(19), Zero) => Done,
                (Command(bit), One) => {
                    self.command |= 1 << bit;
                    Command(bit + 1)
                }
                (Command(bit), Zero) => Command(bit + 1),
                (Command(_), _) => Err(ReceiverError::Data(0)),

                (Done, _) => Done,
                (Err(err), _) => Err(err),
                (Disabled, _) => Disabled,
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
}

const fn nsamples_from_timing(t: &SbpTiming, samplerate: u32) -> [(u32, u32); 4] {
    let per: u32 = 1000 / (samplerate / 1000);
    [
        ((t.hh + t.hl) / per, 5),
        ((t.data + t.paus) / per, 5),
        ((t.data + t.zero) / per, 10),
        ((t.data + t.one) / per, 10),
    ]
}

struct SbpTiming {
    /// Header high
    hh: u32,
    /// Header low
    hl: u32,
    /// Repeat low
    paus: u32,
    /// Data high
    data: u32,
    /// Zero low
    zero: u32,
    /// One low
    one: u32,
}

const TIMING: SbpTiming = SbpTiming {
    hh: 4500,
    hl: 4500,
    paus: 4500,
    data: 500,
    zero: 500,
    one: 1500,
};

#[derive(Debug)]
pub enum SbpPulse {
    Sync = 0,
    Paus = 1,
    Zero = 2,
    One = 3,
    NotAPulseWidth = 4,
}

impl Default for SbpPulse {
    fn default() -> Self {
        SbpPulse::NotAPulseWidth
    }
}

impl From<usize> for SbpPulse {
    fn from(v: usize) -> Self {
        match v {
            0 => SbpPulse::Sync,
            1 => SbpPulse::Paus,
            2 => SbpPulse::Zero,
            3 => SbpPulse::One,
            _ => SbpPulse::NotAPulseWidth,
        }
    }
}
