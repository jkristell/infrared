use core::marker::PhantomData;

use crate::{
    protocol::{
        nec::{NecCommand, NecCommandVariant},
        Nec,
    },
    receiver::{
        time::{InfraMonotonic, PulseSpans},
        DecodingError, ProtocolDecoder, ProtocolDecoderAdaptor, State,
    },
};

impl<Mono: InfraMonotonic, Cmd: NecCommandVariant> ProtocolDecoderAdaptor<Mono> for Nec<Cmd> {
    type Decoder = NecDecoder<Mono, Cmd>;
    const PULSE: [u32; 8] = [
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.header_low,
        Cmd::PULSE_DISTANCE.header_high + Cmd::PULSE_DISTANCE.repeat_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_zero_low,
        Cmd::PULSE_DISTANCE.data_high + Cmd::PULSE_DISTANCE.data_one_low,
        0,
        0,
        0,
        0,
    ];

    const TOL: [u32; 8] = [7, 7, 5, 5, 0, 0, 0, 0];

    fn decoder(freq: u32) -> Self::Decoder {
        NecDecoder {
            state: NecState::Init,
            bitbuf: 0,
            cmd_type: Default::default(),
            dt_save: Mono::ZERO_DURATION,
            pulsespans: <Self as ProtocolDecoderAdaptor<Mono>>::create_pulsespans(freq),
        }
    }
}

pub struct NecDecoder<Mono: InfraMonotonic, C = NecCommand> {
    // State
    state: NecState,
    // Data buffer
    bitbuf: u32,
    // Nec Command type
    cmd_type: PhantomData<C>,
    // Saved dt
    dt_save: Mono::Duration,

    pulsespans: PulseSpans<Mono::Duration>,
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

impl From<NecState> for State {
    fn from(ns: NecState) -> Self {
        use NecState::*;
        match ns {
            Init => State::Idle,
            Done | RepeatDone => State::Done,
            Err(e) => State::Error(e),
            _ => State::Receiving,
        }
    }
}

impl<Cmd, Mono> ProtocolDecoder<Mono, Cmd> for NecDecoder<Mono, Cmd>
where
    Cmd: NecCommandVariant,
    Mono: InfraMonotonic,
{
    #[rustfmt::skip]
    fn event(
        &mut self,
        rising: bool,
        dur: Mono::Duration,
    ) -> State {

        use NecState::*;
        use PulseWidth::*;

        if rising {

            let total_duration = dur + self.dt_save;

            let pulsewidth = self.pulsespans.get(total_duration)
                .unwrap_or(PulseWidth::Invalid);

            let status = match (self.state, pulsewidth) {
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
                "State(prev, new): ({:?}, {:?}) pulsewidth: {:?}",
                self.state,
                status,
                pulsewidth
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
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    Invalid = 4,
}

impl From<usize> for PulseWidth {
    fn from(v: usize) -> Self {
        match v {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Repeat,
            2 => PulseWidth::Zero,
            3 => PulseWidth::One,
            _ => PulseWidth::Invalid,
        }
    }
}
