//! # Samsung36
//!
//! Protocol used by Samsung Blu-ray players and other Samsung devices. Known
//! as Samsung36 in IRremote and the IRP protocol database.
//!
//! Pulse distance coding is used. After the header the 16 bit address is sent.
//! Then a pause and 4 bits of unknown function (sent as zero), followed by
//! the 8 bit command, sent twice, the second time inverted.

mod encoder;
#[cfg(test)]
mod tests;

use crate::{
    protocol::{Command, Protocol, ProtocolId},
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderBuilder, DecodingError, ProtocolDecoder, DecoderState,
    },
};

/// Samsung36 protocol
pub struct Samsung36;

impl Protocol for Samsung36 {
    type Cmd = Samsung36Command;
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

impl<Mono: InfraMonotonic> DecoderBuilder<Mono> for Samsung36 {
    type Decoder = Samsung36Decoder<Mono>;

    fn build(freq: u32) -> Self::Decoder {
        Samsung36Decoder {
            state: Samsung36State::Init,
            address: 0,
            command: 0,
            since_rising: Mono::ZERO_DURATION,
            spans: PulseSpans::new(freq, &PULSE, &TOL),
        }
    }
}

/// Samsung36 decoder
pub struct Samsung36Decoder<Mono: InfraMonotonic> {
    state: Samsung36State,
    address: u16,
    command: u32,
    since_rising: Mono::Duration,
    spans: PulseSpans<Mono>,
}

/// Samsung36 command, 16 bit address and 8 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Samsung36Command {
    /// Address
    pub address: u16,
    /// Command
    pub command: u8,
}

impl Samsung36Command {
    /// Create a new command
    pub const fn new(address: u16, command: u8) -> Self {
        Self { address, command }
    }

    /// The 20 bits after the pause: 4 unknown bits (zero), the command and
    /// the inverted command
    const fn command_bits(&self) -> u32 {
        ((self.command as u32) | ((!self.command as u32) << 8)) << 4
    }
}

/// Check the inverted command copy of the 20 bits after the pause
const fn command_bits_valid(bits: u32) -> bool {
    let bits = bits >> 4;
    ((bits ^ (bits >> 8)) & 0xFF) == 0xFF
}

impl Command for Samsung36Command {
    const PROTOCOL: ProtocolId = ProtocolId::Samsung36;

    fn address(&self) -> u32 {
        self.address.into()
    }

    fn command(&self) -> u32 {
        self.command.into()
    }

    fn is_repeat(&self) -> bool {
        false
    }

    fn raw(&self) -> u64 {
        u64::from(self.address) | (u64::from(self.command_bits()) << 16)
    }

    fn from_parts(address: u32, command: u32) -> Option<Self> {
        Some(Samsung36Command::new(
            address.try_into().ok()?,
            command.try_into().ok()?,
        ))
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// Internal receiver state
pub enum Samsung36State {
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

impl<Mono: InfraMonotonic> ProtocolDecoder<Samsung36, Mono> for Samsung36Decoder<Mono> {
    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: Mono::Duration) -> DecoderState {
        use Samsung36Symbol::*;
        use Samsung36State::*;

        if rising {
            let dt = self.since_rising + dt;
            let pulsewidth = self.spans.get::<Samsung36Symbol>(dt).unwrap_or(Samsung36Symbol::Invalid);

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

                (Command(19),   One)    => { self.command |= 1 << 19; self.check() }
                (Command(19),   Zero)   => self.check(),
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
    fn command(&self) -> Option<Samsung36Command> {
        Some(Samsung36Command::new(self.address, (self.command >> 4) as u8))
    }

    fn reset(&mut self) {
        self.state = Samsung36State::Init;
        self.address = 0;
        self.command = 0;
        self.since_rising = Mono::ZERO_DURATION;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.spans
    }
}

impl<Mono: InfraMonotonic> Samsung36Decoder<Mono> {
    fn check(&self) -> Samsung36State {
        if command_bits_valid(self.command) {
            Samsung36State::Done
        } else {
            Samsung36State::Err(DecodingError::Validation)
        }
    }
}

impl From<Samsung36State> for DecoderState {
    fn from(state: Samsung36State) -> DecoderState {
        use Samsung36State::*;
        match state {
            Init => DecoderState::Idle,
            Done => DecoderState::Done,
            Err(e) => DecoderState::Error(e),
            _ => DecoderState::Receiving,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Samsung36Symbol {
    Sync = 0,
    Pause = 1,
    Zero = 2,
    One = 3,
    Invalid = 4,
}

impl From<usize> for Samsung36Symbol {
    fn from(v: usize) -> Self {
        match v {
            0 => Samsung36Symbol::Sync,
            1 => Samsung36Symbol::Pause,
            2 => Samsung36Symbol::Zero,
            3 => Samsung36Symbol::One,
            _ => Samsung36Symbol::Invalid,
        }
    }
}
