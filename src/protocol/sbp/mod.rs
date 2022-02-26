//! # Samsung Blu-Ray Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used with. After the Header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use core::convert::TryInto;

use crate::receiver::time::InfraMonotonic;
#[cfg(feature = "remotes")]
use crate::receiver::{DecoderStateMachine, DecodingError, Status};
use crate::{
    cmd::{AddressCommand, Command},
    protocol::Protocol,
    receiver::{time::PulseSpans, DecoderState},
};

/// Samsung BluRay player protocol
pub struct Sbp;

impl Protocol for Sbp {
    type Cmd = SbpCommand;
}

/// Samsung Blu-ray protocol
pub struct SbpReceiverState<T: InfraMonotonic> {
    state: SbpStatus,
    address: u16,
    command: u32,
    since_rising: T::Duration,
}

impl<T: InfraMonotonic> DecoderState for SbpReceiverState<T> {
    fn reset(&mut self) {
        self.state = SbpStatus::Init;
        self.address = 0;
        self.command = 0;
        self.since_rising = T::ZERO_DURATION;
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl Command for SbpCommand {
    fn is_repeat(&self) -> bool {
        false
    }
}

impl AddressCommand for SbpCommand {
    fn address(&self) -> u32 {
        self.address.into()
    }

    fn command(&self) -> u32 {
        self.command.into()
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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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

impl<Time: InfraMonotonic> DecoderStateMachine<Time> for Sbp {
    type State = SbpReceiverState<Time>;
    type InternalStatus = SbpStatus;
    const PULSE_LENGTHS: [u32; 8] = [
        (4500 + 4500),
        (500 + 4500),
        (500 + 500),
        (500 + 1500),
        0,
        0,
        0,
        0,
    ];
    const TOLERANCE: [u32; 8] = [5, 5, 10, 10, 0, 0, 0, 0];

    fn state() -> Self::State {
        SbpReceiverState {
            state: SbpStatus::Init,
            address: 0,
            command: 0,
            since_rising: Time::ZERO_DURATION,
        }
    }

    #[rustfmt::skip]
    fn new_event(state: &mut Self::State, ranges: &PulseSpans<Time::Duration>, rising: bool, dt: Time::Duration)

                 -> SbpStatus {
        use SbpPulse::*;
        use SbpStatus::*;

        if rising {
            let dt = state.since_rising + dt;
            let pulsewidth = Time::find::<SbpPulse>(ranges, dt).unwrap_or(SbpPulse::NotAPulseWidth);

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

/*
struct SbpTiming {
    /// Header high
    hh: u32,
    /// Header low
    hl: u32,
    /// Repeat low
    pause: u32,
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
    pause: 4500,
    data: 500,
    zero: 500,
    one: 1500,
};
*/

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
