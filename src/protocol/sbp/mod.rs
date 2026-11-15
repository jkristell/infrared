//! # Samsung Blu-Ray Player Protocol
//!
//! Protocol used on some Samsung BluRay players and probably other devices from Samsung.
//!
//! Pulse distance coding is used. After the header the 16 bit address is sent.
//! Then a pause and then 4 bits of unknown function (could be repeat indicator?)
//! After this the 8 bit command is sent twice, second time inverted.
//!

use core::convert::TryInto;

mod encoder;
#[cfg(test)]
mod tests;

use crate::{
    cmd::{AddressedCommand, Command},
    protocol::Protocol,
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderBuilder, DecodingError, ProtocolDecoder, DecoderState,
    },
    ProtocolId,
};

/// Samsung BluRay player protocol
pub struct Sbp;

impl Protocol for Sbp {
    type Cmd = SbpCommand;
}

const HEADER_MARK: u32 = 4500;
const HEADER_SPACE: u32 = 4500;
const DATA_MARK: u32 = 500;
const PAUSE_SPACE: u32 = 4500;
const ZERO_SPACE: u32 = 500;
const ONE_SPACE: u32 = 1500;

const PULSE: [u32; 8] = [
    (HEADER_MARK + HEADER_SPACE),
    (DATA_MARK + PAUSE_SPACE),
    (DATA_MARK + ZERO_SPACE),
    (DATA_MARK + ONE_SPACE),
    0,
    0,
    0,
    0,
];
const TOL: [u32; 8] = [5, 5, 10, 10, 0, 0, 0, 0];

impl<Mono: InfraMonotonic> DecoderBuilder<Mono> for Sbp {
    type Decoder = SbpDecoder<Mono>;

    fn build(freq: u32) -> Self::Decoder {
        SbpDecoder {
            state: SbpState::Init,
            address: 0,
            command: 0,
            since_rising: Mono::ZERO_DURATION,
            spans: PulseSpans::new(freq, &PULSE, &TOL),
        }
    }
}

/// Samsung Blu-ray protocol
pub struct SbpDecoder<Mono: InfraMonotonic> {
    state: SbpState,
    address: u16,
    command: u32,
    since_rising: Mono::Duration,
    spans: PulseSpans<Mono>,
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

impl AddressedCommand for SbpCommand {
    const ID: ProtocolId = ProtocolId::Sbp;

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
    Err(DecodingError),
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Sbp, Mono> for SbpDecoder<Mono> {
    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: Mono::Duration) -> DecoderState {
        use SbpSymbol::*;
        use SbpState::*;

        if rising {
            let dt = self.since_rising + dt;
            let pulsewidth = self.spans.get::<SbpSymbol>(dt).unwrap_or(SbpSymbol::Invalid);

            self.state = match (self.state, pulsewidth) {
                (Init,          Sync)   => Address(0),
                (Init,          _)      => Init,

                (Address(15),   One)    => { self.address |= 1 << 15; Divider }
                (Address(15),   Zero)   => Divider,
                (Address(bit),  One)    => { self.address |= 1 << bit; Address(bit + 1) }
                (Address(bit),  Zero)   => Address(bit + 1),
                (Address(_),    _)      => Err(DecodingError::Address),

                (Divider,       Pause)   => Command(0),
                (Divider,       _)      => Err(DecodingError::Data),

                (Command(19),   One)    => { self.command |= 1 << 19; Done }
                (Command(19),   Zero)   => Done,
                (Command(bit),  One)    => { self.command |= 1 << bit; Command(bit + 1) }
                (Command(bit),  Zero)   => Command(bit + 1),
                (Command(_),    _)      => Err(DecodingError::Data),

                (Done,          _)      => Done,
                (Err(err),      _)      => Err(err),
            };
        } else {
            self.since_rising = dt;
        }

        self.state.into()
    }
    fn command(&self) -> Option<SbpCommand> {
        Some(SbpCommand::unpack(self.address, self.command))
    }

    fn reset(&mut self) {
        self.state = SbpState::Init;
        self.address = 0;
        self.command = 0;
        self.since_rising = Mono::ZERO_DURATION;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.spans
    }
}

impl From<SbpState> for DecoderState {
    fn from(state: SbpState) -> DecoderState {
        use SbpState::*;
        match state {
            Init => DecoderState::Idle,
            Done => DecoderState::Done,
            Err(e) => DecoderState::Error(e),
            _ => DecoderState::Receiving,
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
pub enum SbpSymbol {
    Sync = 0,
    Pause = 1,
    Zero = 2,
    One = 3,
    Invalid = 4,
}

impl From<usize> for SbpSymbol {
    fn from(v: usize) -> Self {
        match v {
            0 => SbpSymbol::Sync,
            1 => SbpSymbol::Pause,
            2 => SbpSymbol::Zero,
            3 => SbpSymbol::One,
            _ => SbpSymbol::Invalid,
        }
    }
}
