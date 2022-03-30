use core::marker::PhantomData;

use crate::receiver::time::{InfraMonotonic, PulseSpans};
use crate::{
    protocol::{
        nec::{NecCommand, NecCommandVariant},
        Nec,
    },
    receiver::{DecoderData, Decoder, DecodingError, State},
};

pub struct NecData<Mono: InfraMonotonic, C = NecCommand> {
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

impl<C: NecCommandVariant, Mono: InfraMonotonic> DecoderData for NecData<Mono, C> {
    fn reset(&mut self) {
        self.state = NecState::Init;
        self.dt_save = Mono::ZERO_DURATION;
    }
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

impl<Cmd, Mono> Decoder<Mono> for Nec<Cmd>
where
    Cmd: NecCommandVariant,
    Mono: InfraMonotonic,
{
    type Data = NecData<Mono, Cmd>;
    type InternalState = NecState;

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

    fn decoder(freq: u32) -> Self::Data {
        NecData {
            state: NecState::Init,
            bitbuf: 0,
            cmd_type: Default::default(),
            dt_save: Mono::ZERO_DURATION,
            pulsespans: <Self as Decoder<Mono>>::create_pulsespans(freq),
        }
    }

    #[rustfmt::skip]
    fn event(
        self_: &mut Self::Data,
        rising: bool,
        dur: Mono::Duration,
    ) -> Self::InternalState {

        use NecState::*;
        use PulseWidth::*;

        if rising {

            let total_duration = dur + self_.dt_save;

            let pulsewidth = Mono::find::<PulseWidth>(&self_.pulsespans, total_duration)
                .unwrap_or(PulseWidth::Invalid);


            let status = match (self_.state, pulsewidth) {
                (Init,              Sync)   => { self_.bitbuf = 0; Receiving(0) },
                (Init,              Repeat) => RepeatDone,
                (Init,              _)      => Init,

                (Receiving(31),     One)    => { self_.bitbuf |= 1 << 31; Done }
                (Receiving(31),     Zero)   => Done,
                (Receiving(bit),    One)    => { self_.bitbuf |= 1 << bit; Receiving(bit + 1) }
                (Receiving(bit),    Zero)   => Receiving(bit + 1),
                (Receiving(_),      _)      => Err(DecodingError::Data),

                (Done,              _)      => Done,
                (RepeatDone,        _)      => RepeatDone,
                (Err(err),          _)      => Err(err),
            };

            trace!(
                "State(prev, new): ({:?}, {:?}) pulsewidth: {:?}",
                self_.state,
                status,
                pulsewidth
            );

            self_.state = status;

            self_.dt_save = Mono::ZERO_DURATION;
        } else {
            // Save
            self_.dt_save = dur;
        }

        self_.state
    }

    fn command(self_: &Self::Data) -> Option<Self::Cmd> {
        match self_.state {
            NecState::Done => Self::Cmd::unpack(self_.bitbuf, false),
            NecState::RepeatDone => Self::Cmd::unpack(self_.bitbuf, true),
            _ => None,
        }
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
