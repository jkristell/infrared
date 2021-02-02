//! # Samsung Blu-Ray Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used with. After the Header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use core::convert::TryInto;

use crate::{
    protocols::utils::PulseWidthRange,
    recv::{Error, InfraredReceiver, State},
    ProtocolId,
};
#[cfg(feature = "remotes")]
use crate::remotecontrol::AsButton;

#[derive(Debug)]
/// Samsung Blu-ray protocol
pub struct Sbp {
    state: SbpState,
    address: u16,
    command: u32,
    since_rising: u32,
    ranges: PulseWidthRange<SbpPulse>,
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
        let valid = ((command ^ (command >> 8)) & 0xFF) == 0xFF;

        Self {
            address,
            command: (command) as u8,
            valid,
        }
    }
}

#[cfg(feature = "remotes")]
impl AsButton for SbpCommand {
    fn address(&self) -> u32 {
        self.address.into()
    }

    fn command(&self) -> u32 {
        self.command.into()
    }

    fn protocol(&self) -> ProtocolId {
        ProtocolId::Sbp
    }

    fn create(address: u32, command: u32) -> Option<Self> {
        Some(SbpCommand {
            address: address.try_into().ok()?,
            command: command.try_into().ok()?,
            valid: true,
        })
    }
}

#[derive(Debug, Copy, Clone)]
// Internal receiver state
pub enum SbpState {
    // Waiting for first pulse
    Init,
    // Receiving address
    Address(u16),
    /// Pause
    Divider,
    // Receiving data
    Command(u16),
    // Command received
    Done,
    // In error state
    Err(Error),
}

impl InfraredReceiver for Sbp {
    type Cmd = SbpCommand;
    type InternalState = SbpState;

    fn create() -> Self {
        Self::default()
    }

    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: u32) -> SbpState {
        use SbpPulse::*;
        use SbpState::*;

        if rising {
            let dt = self.since_rising + dt;
            let pulsewidth = self.ranges.pulsewidth(dt);

            self.state = match (self.state, pulsewidth) {
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
                (Address(_), _) => Err(Error::Address),

                (Divider, Paus) => Command(0),
                (Divider, _) => Err(Error::Data),

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
                (Command(_), _) => Err(Error::Data),

                (Done, _) => Done,
                (Err(err), _) => Err(err),
            };
        } else {
            self.since_rising = dt;
        }

        self.state
    }

    fn command(&self) -> Option<Self::Cmd> {
        Some(SbpCommand::from_receiver(self.address, self.command))
    }

    fn reset(&mut self) {
        self.state = SbpState::Init;
        self.address = 0;
        self.command = 0;
    }
}

impl Default for SbpState {
    fn default() -> Self {
        Self::Init
    }
}

impl From<SbpState> for State {
    fn from(state: SbpState) -> State {
        use SbpState::*;
        match state {
            Init => State::Idle,
            Done => State::Done,
            Err(e) => State::Error(e),
            _ => State::Receiving,
        }
    }
}

impl Default for Sbp {
    fn default() -> Self {
        let nsamples = nsamples_from_timing(&TIMING);
        let ranges = PulseWidthRange::new(&nsamples);

        Self {
            state: SbpState::Init,
            address: 0,
            command: 0,
            since_rising: 0,
            ranges,
        }
    }
}

const fn nsamples_from_timing(t: &SbpTiming) -> [(u32, u32); 4] {
    [
        ((t.hh + t.hl), 5),
        ((t.data + t.paus), 5),
        ((t.data + t.zero), 10),
        ((t.data + t.one), 10),
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
