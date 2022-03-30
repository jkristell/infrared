use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::Protocol,
    receiver::{DecoderData, Decoder, State},
};

#[cfg(test)]
mod test;

const HEADER_HIGH: u32 = 3400;
const HEADER_LOW: u32 = 1600;
const DATA_HIGH: u32 = 480;
const ZERO_LOW: u32 = 360;
const ONE_LOW: u32 = 1200;

/// Denon protocol
pub struct Denon;

impl Protocol for Denon {
    type Cmd = DenonCommand;
}

pub struct DenonData<Mono: InfraMonotonic> {
    state: DenonState,
    buf: u64,
    dt_save: Mono::Duration,
    spans: PulseSpans<Mono::Duration>
}

impl<T: InfraMonotonic> DecoderData for DenonData<T> {
    fn reset(&mut self) {
        self.state = DenonState::Idle;
        self.buf = 0;
        self.dt_save = T::ZERO_DURATION;
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DenonCommand {
    pub bits: u64,
}

impl<Mono: InfraMonotonic> Decoder<Mono> for Denon {
    type Data = DenonData<Mono>;
    type InternalState = DenonState;

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

    fn decoder(freq: u32) -> Self::Data {
        DenonData {
            state: DenonState::Idle,
            buf: 0,
            dt_save: Mono::ZERO_DURATION,
            spans: <Self as Decoder<Mono>>::create_pulsespans(freq),
        }
    }

    #[rustfmt::skip]
    fn event(self_: &mut Self::Data, rising: bool, dt: Mono::Duration) -> DenonState {

        if rising {
            let pulsewidth = self_.spans.get::<PulseWidth>( self_.dt_save + dt)
                .unwrap_or(PulseWidth::Fail);

            self_.state = match (self_.state, pulsewidth) {
                (DenonState::Idle,          PulseWidth::Sync)   => DenonState::Data(0),
                (DenonState::Idle,          _)                  => DenonState::Idle,
                (DenonState::Data(47),      PulseWidth::Zero)   => DenonState::Done,
                (DenonState::Data(47),      PulseWidth::One)    => DenonState::Done,
                (DenonState::Data(idx),     PulseWidth::Zero)   => DenonState::Data(idx + 1),
                (DenonState::Data(idx),     PulseWidth::One)    => { self_.buf |= 1 << idx; DenonState::Data(idx + 1) }
                (DenonState::Data(_ix),     _)                  => DenonState::Idle,
                (DenonState::Done,          _)                  => DenonState::Done,
            };

            self_.dt_save = Mono::ZERO_DURATION;
        } else {
            self_.dt_save = dt;
        }

        self_.state
    }
    fn command(state: &Self::Data) -> Option<Self::Cmd> {
        if state.state == DenonState::Done {
            Some(DenonCommand { bits: state.buf })
        } else {
            None
        }
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
