use core::marker::PhantomData;

use crate::{
    Level,
    protocol::{
        nec::{NecCommand, NecCommandVariant},
        Nec,
    },
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecoderBuilder, DecodingError, ProtocolDecoder, DecoderState,
    },
};

const fn pulselens<Cmd: NecCommandVariant>() -> [u32; 8] {
    [
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.header_low,
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.repeat_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_zero_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_one_low,
        0,
        0,
        0,
        0,
    ]
}

const TOL: [u32; 8] = [7, 7, 25, 12, 0, 0, 0, 0];

impl<Mono: InfraMonotonic, Cmd: NecCommandVariant> DecoderBuilder<Mono> for Nec<Cmd> {
    type Decoder = NecDecoder<Mono, Cmd>;

    fn build(freq: u32) -> Self::Decoder {
        NecDecoder {
            state: NecState::Init,
            bitbuf: 0,
            cmd_type: Default::default(),
            dt_save: Mono::ZERO_DURATION,
            pulsespans: PulseSpans::new(freq, &pulselens::<Cmd>(), &TOL),
        }
    }
}

pub struct NecDecoder<Mono: InfraMonotonic, C = NecCommand> {
    // DecoderState
    state: NecState,
    // Data buffer
    bitbuf: u32,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: Mono::Duration,

    pulsespans: PulseSpans<Mono>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
// Internal receiver state
pub enum NecState {
    // Waiting for first pulse
    Init,
    // Receiving data
    Receiving(u32),
    // Command received
    Done,
    // Repeat command received
    RepeatDone,
    // In error state
    Err(DecodingError),
}

impl From<NecState> for DecoderState {
    fn from(ns: NecState) -> Self {
        use NecState::*;
        match ns {
            Init => DecoderState::Idle,
            Done | RepeatDone => DecoderState::Done,
            Err(e) => DecoderState::Error(e),
            _ => DecoderState::Receiving,
        }
    }
}

impl<Mono, Cmd> ProtocolDecoder<Nec<Cmd>, Mono> for NecDecoder<Mono, Cmd>
where
    Mono: InfraMonotonic,
    Cmd: NecCommandVariant,
{
    #[rustfmt::skip]
    fn event(&mut self, dur: Mono::Duration, level: Level) -> DecoderState {
        let mark = level.is_mark();

        use NecState::*;
        use NecSymbol::*;

        if mark {

            let total_duration = dur + self.dt_save;

            let symbol = self.pulsespans.get(total_duration)
                .unwrap_or(NecSymbol::Invalid);

            let status = match (self.state, symbol) {
                (Init,              Sync)   => { self.bitbuf = 0; Receiving(0) },
                (Init,              Repeat) => RepeatDone,
                (Init,              _)      => Init,

                (Receiving(31),     One)    => { self.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { self.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(DecodingError::Data),

                (Done,              _)      => Done,
                (RepeatDone,        _)      => RepeatDone,
                (Err(err),          _)      => Err(err),
            };

            trace!(
                "DecoderState(prev, new): ({:?}, {:?}) symbol: {:?}",
                self.state,
                status,
                symbol
            );

            self.state = status;

            self.dt_save = Mono::ZERO_DURATION;
        } else {
            // Save
            self.dt_save = dur;
        }

        self.state.into()
    }

    fn command(&self) -> Option<Cmd> {
        match self.state {
            NecState::Done => Cmd::unpack(self.bitbuf, false),
            NecState::RepeatDone => Cmd::unpack(self.bitbuf, true),
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.state = NecState::Init;
        self.dt_save = Mono::ZERO_DURATION;
    }

    fn spans(&self) -> &PulseSpans<Mono> {
        &self.pulsespans
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NecSymbol {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    Invalid = 4,
}

impl From<usize> for NecSymbol {
    fn from(v: usize) -> Self {
        match v {
            0 => NecSymbol::Sync,
            1 => NecSymbol::Repeat,
            2 => NecSymbol::Zero,
            3 => NecSymbol::One,
            _ => NecSymbol::Invalid,
        }
    }
}
