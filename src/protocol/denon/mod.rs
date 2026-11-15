//! # Denon-K
//!
//! The Denon member of the Kaseikyo (Panasonic/Japanese) protocol family.
//! 48 bit frames, sent LSB first:
//!
//! | Bits   | Content                                     |
//! |--------|---------------------------------------------|
//! | 0..16  | Vendor id, 0x3254 for Denon                 |
//! | 16..20 | Vendor parity, zero for Denon               |
//! | 20..32 | Address                                     |
//! | 32..40 | Command                                     |
//! | 40..48 | Parity, the XOR of the bytes 2, 3 and 4     |
//!
//! The address and command split is the same as IRremote's `Kaseikyo_Denon`.

use crate::{
    protocol::{Command, Protocol, ProtocolId},
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderBuilder, DecodingError, ProtocolDecoder, DecoderState,
    },
};

mod encoder;
#[cfg(test)]
mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;
const PULSE: [u32; 8] = [
    (HEADER_HIGH + HEADER_LOW),
    (DATA_HIGH + ZERO_LOW),
    (DATA_HIGH + ONE_LOW),
    0,
    0,
    0,
    0,
    0,
];

const TOL: [u32; 8] = [8, 10, 10, 0, 0, 0, 0, 0];

/// Denon-K protocol
pub struct Denon;

impl Protocol for Denon {
    type Cmd = DenonCommand;
}

impl<Mono: InfraMonotonic> DecoderBuilder<Mono> for Denon {
    type Decoder = DenonDecoder<Mono>;

    fn build(freq: u32) -> Self::Decoder {
        DenonDecoder {
            state: DenonState::Idle,
            buf: 0,
            dt_save: Mono::ZERO_DURATION,
            spans: PulseSpans::new(freq, &PULSE, &TOL),
        }
    }
}

/// Denon-K decoder
pub struct DenonDecoder<Mono: InfraMonotonic> {
    state: DenonState,
    buf: u64,
    dt_save: Mono::Duration,
    spans: PulseSpans<Mono>,
}

const VENDOR: u64 = 0x3254;

/// Denon-K command, 12 bit address and 8 bit command
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DenonCommand {
    /// Address, 12 bits
    pub address: u16,
    /// Command
    pub command: u8,
}

impl DenonCommand {
    /// Create a new command. Only the lower 12 bits of the address are used.
    pub const fn new(address: u16, command: u8) -> Self {
        Self { address, command }
    }

    /// Pack the command into the 48 bits of a frame
    pub const fn pack(&self) -> u64 {
        let data = ((self.address as u64 & 0xFFF) << 4) | ((self.command as u64) << 16);
        let parity = (data ^ (data >> 8) ^ (data >> 16)) & 0xFF;
        VENDOR | (data << 16) | (parity << 40)
    }

    /// Unpack the 48 bits of a frame
    pub const fn unpack(bits: u64) -> Result<Self, DecodingError> {
        // Vendor id and vendor parity
        if bits & 0xF_FFFF != VENDOR {
            return Err(DecodingError::Address);
        }
        let data = bits >> 16;
        if (data ^ (data >> 8) ^ (data >> 16) ^ (data >> 24)) & 0xFF != 0 {
            return Err(DecodingError::Validation);
        }
        Ok(DenonCommand {
            address: ((bits >> 20) & 0xFFF) as u16,
            command: ((bits >> 32) & 0xFF) as u8,
        })
    }
}

impl Command for DenonCommand {
    const PROTOCOL: ProtocolId = ProtocolId::Denon;

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
        self.pack()
    }

    fn from_parts(address: u32, command: u32) -> Option<Self> {
        if address >= 1 << 12 {
            return None;
        }
        Some(DenonCommand::new(address as u16, command.try_into().ok()?))
    }
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Denon, Mono> for DenonDecoder<Mono> {
    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: Mono::Duration) -> DecoderState {

        if rising {
            let symbol = self.spans.get::<DenonSymbol>( self.dt_save + dt)
                .unwrap_or(DenonSymbol::Fail);

            self.state = match (self.state, symbol) {
                (DenonState::Idle,          DenonSymbol::Sync)   => DenonState::Data(0),
                (DenonState::Idle,          _)                  => DenonState::Idle,
                (DenonState::Data(47),      DenonSymbol::Zero)   => self.check(),
                (DenonState::Data(47),      DenonSymbol::One)    => { self.buf |= 1 << 47; self.check() }
                (DenonState::Data(idx),     DenonSymbol::Zero)   => DenonState::Data(idx + 1),
                (DenonState::Data(idx),     DenonSymbol::One)    => { self.buf |= 1 << idx; DenonState::Data(idx + 1) }
                (DenonState::Data(_ix),     _)                  => DenonState::Idle,
                (DenonState::Done,          _)                  => DenonState::Done,
                (DenonState::Err(err),      _)                  => DenonState::Err(err),
            };

            self.dt_save = Mono::ZERO_DURATION;
        } else {
            self.dt_save = dt;
        }

        self.state.into()
    }

    fn command(&self) -> Option<DenonCommand> {
        if self.state == DenonState::Done {
            DenonCommand::unpack(self.buf).ok()
        } else {
            None
        }
    }
    fn reset(&mut self) {
        self.state = DenonState::Idle;
        self.buf = 0;
        self.dt_save = Mono::ZERO_DURATION;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.spans
    }
}

impl<Mono: InfraMonotonic> DenonDecoder<Mono> {
    fn check(&self) -> DenonState {
        match DenonCommand::unpack(self.buf) {
            Ok(_) => DenonState::Done,
            Err(err) => DenonState::Err(err),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DenonState {
    Idle,
    Data(u8),
    Done,
    Err(DecodingError),
}

impl From<DenonState> for DecoderState {
    fn from(status: DenonState) -> Self {
        match status {
            DenonState::Idle => DecoderState::Idle,
            DenonState::Data(_) => DecoderState::Receiving,
            DenonState::Done => DecoderState::Done,
            DenonState::Err(err) => DecoderState::Error(err),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum DenonSymbol {
    Sync,
    Zero,
    One,
    Fail,
}

impl From<usize> for DenonSymbol {
    fn from(value: usize) -> Self {
        match value {
            0 => DenonSymbol::Sync,
            1 => DenonSymbol::Zero,
            2 => DenonSymbol::One,
            _ => DenonSymbol::Fail,
        }
    }
}
