use crate::{
    protocol::Protocol,
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderBuilder, ProtocolDecoder, State,
    },
};

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

/// Denon protocol
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

pub struct DenonDecoder<Mono: InfraMonotonic> {
    state: DenonState,
    buf: u64,
    dt_save: Mono::Duration,
    spans: PulseSpans<Mono>,
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DenonCommand {
    pub bits: u64,
}

impl<Mono: InfraMonotonic> ProtocolDecoder<Denon, Mono> for DenonDecoder<Mono> {
    #[rustfmt::skip]
    fn event(&mut self, rising: bool, dt: Mono::Duration) -> State {

        if rising {
            let pulsewidth = self.spans.get::<PulseWidth>( self.dt_save + dt)
                .unwrap_or(PulseWidth::Fail);

            self.state = match (self.state, pulsewidth) {
                (DenonState::Idle,          PulseWidth::Sync)   => DenonState::Data(0),
                (DenonState::Idle,          _)                  => DenonState::Idle,
                (DenonState::Data(47),      PulseWidth::Zero)   => DenonState::Done,
                (DenonState::Data(47),      PulseWidth::One)    => DenonState::Done,
                (DenonState::Data(idx),     PulseWidth::Zero)   => DenonState::Data(idx + 1),
                (DenonState::Data(idx),     PulseWidth::One)    => { self.buf |= 1 << idx; DenonState::Data(idx + 1) }
                (DenonState::Data(_ix),     _)                  => DenonState::Idle,
                (DenonState::Done,          _)                  => DenonState::Done,
            };

            self.dt_save = Mono::ZERO_DURATION;
        } else {
            self.dt_save = dt;
        }

        self.state.into()
    }

    fn command(&self) -> Option<DenonCommand> {
        if self.state == DenonState::Done {
            Some(DenonCommand { bits: self.buf })
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DenonState {
    Idle,
    Data(u8),
    Done,
}

impl From<DenonState> for State {
    fn from(status: DenonState) -> Self {
        match status {
            DenonState::Idle => State::Idle,
            DenonState::Data(_) => State::Receiving,
            DenonState::Done => State::Done,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum PulseWidth {
    Sync,
    Zero,
    One,
    Fail,
}

impl From<usize> for PulseWidth {
    fn from(value: usize) -> Self {
        match value {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Zero,
            2 => PulseWidth::One,
            _ => PulseWidth::Fail,
        }
    }
}
