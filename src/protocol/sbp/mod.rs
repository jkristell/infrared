//! # Samsung Blu-Ray Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used with. After the Header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use core::convert::TryInto;

use crate::protocol::utils::InfraConstRange;
use crate::protocol::Protocol;
use crate::receiver::DecoderState;
#[cfg(feature = "remotes")]
use crate::remotecontrol::AsButton;
use crate::{
    receiver::{DecoderStateMachine, DecodingError, Status},
    ProtocolId,
};

/// Samsung BluRay player protocol
pub struct Sbp;

impl Protocol for Sbp {
    type Cmd = SbpCommand;
}

/// Samsung Blu-ray protocol
pub struct SbpReceiverState {
    state: SbpStatus,
    address: u16,
    command: u32,
    since_rising: usize,
}

impl DecoderState for SbpReceiverState {
    fn reset(&mut self) {
        self.state = SbpStatus::Init;
        self.address = 0;
        self.command = 0;
    }
}

#[derive(Debug)]
pub struct SbpCommand {
    pub address: u16,
    pub command: u8,
    pub valid: bool,
}

impl SbpCommand {
    pub fn unpack(address: u16, mut command: u32) -> Self {
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
pub enum SbpStatus {
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
    Err(DecodingError),
}

impl DecoderStateMachine for Sbp {
    type State = SbpReceiverState;
    type RangeData = InfraConstRange<4>;
    type InternalStatus = SbpStatus;

    fn state() -> Self::State {
        SbpReceiverState {
            state: SbpStatus::Init,
            address: 0,
            command: 0,
            since_rising: 0,
        }
    }

    fn ranges(resolution: usize) -> Self::RangeData {
        let nsamples = nsamples_from_timing(&TIMING);
        InfraConstRange::<4>::new(&nsamples, resolution)
    }

    #[rustfmt::skip]
    fn event_full(state: &mut Self::State, ranges: &Self::RangeData, rising: bool, dt: usize) -> SbpStatus {
        use SbpPulse::*;
        use SbpStatus::*;

        if rising {
            let dt = state.since_rising + dt;
            let pulsewidth = ranges.find::<SbpPulse>(dt).unwrap_or(SbpPulse::NotAPulseWidth);

            state.state = match (state.state, pulsewidth) {
                (Init,          Sync)   => Address(0),
                (Init,          _)      => Init,

                (Address(15),   One)    => { state.address |= 1 << 15; Divider }
                (Address(15),   Zero)   => Divider,
                (Address(bit),  One)    => { state.address |= 1 << bit; Address(bit + 1) }
                (Address(bit),  Zero)   => Address(bit + 1),
                (Address(_),    _)      => Err(DecodingError::Address),

                (Divider,       Paus)   => Command(0),
                (Divider,       _)      => Err(DecodingError::Data),

                (Command(19),   One)    => { state.command |= 1 << 19; Done }
                (Command(19),   Zero)   => Done,
                (Command(bit),  One)    => { state.command |= 1 << bit; Command(bit + 1) }
                (Command(bit),  Zero)   => Command(bit + 1),
                (Command(_),    _)      => Err(DecodingError::Data),

                (Done,          _)      => Done,
                (Err(err),      _)      => Err(err),
            };
        } else {
            state.since_rising = dt;
        }

        state.state
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(SbpCommand::unpack(state.address, state.command))
    }
}

impl From<SbpStatus> for Status {
    fn from(state: SbpStatus) -> Status {
        use SbpStatus::*;
        match state {
            Init => Status::Idle,
            Done => Status::Done,
            Err(e) => Status::Error(e),
            _ => Status::Receiving,
        }
    }
}

const fn nsamples_from_timing(t: &SbpTiming) -> [(usize, usize); 4] {
    [
        ((t.hh + t.hl), 5),
        ((t.data + t.pause), 5),
        ((t.data + t.zero), 10),
        ((t.data + t.one), 10),
    ]
}

struct SbpTiming {
    /// Header high
    hh: usize,
    /// Header low
    hl: usize,
    /// Repeat low
    pause: usize,
    /// Data high
    data: usize,
    /// Zero low
    zero: usize,
    /// One low
    one: usize,
}

const TIMING: SbpTiming = SbpTiming {
    hh: 4500,
    hl: 4500,
    pause: 4500,
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
