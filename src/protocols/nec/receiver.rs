use core::ops::Range;

use crate::nec::Pulsedistance;
use crate::{ReceiverStateMachine, ReceiverState, ProtocolId};
#[cfg(feature = "protocol-dev")]
use crate::ReceiverDebug;
use crate::protocols::nec::{NecTypeTrait, NecCommand};
use crate::receiver::{ReceiverError, ReceiverHal};



pub struct NecTypeReceiver<NECTYPE> {
    // State
    state: NecState,
    last_event: u32,
    pub bitbuf: u32,
    // Timing and tolerances
    tolerance: Tolerances,
    lastcommand: u32,
    nectype: core::marker::PhantomData<NECTYPE>,

    #[cfg(feature = "protocol-dev")]
    pub debug: ReceiverDebug<NecState, Tolerances>,
}

#[derive(Debug, Copy, Clone)]
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
    Err(ReceiverError),
    // Disabled
    Disabled,
}

impl<NECTYPE: NecTypeTrait> NecTypeReceiver<NECTYPE> {
    pub fn new(samplerate: u32) -> Self {
        let timing = &NECTYPE::PULSEDISTANCE;
        Self::new_from_timing(samplerate, timing)
    }

    fn new_from_timing(samplerate: u32, timing: &Pulsedistance) -> Self {
        let tol = Tolerances::from_timing(timing, samplerate);
        Self {
            state: NecState::Init,
            last_event: 0,
            bitbuf: 0,
            lastcommand: 0,
            nectype: core::marker::PhantomData,
            #[cfg(feature = "protocol-dev")]
            debug: ReceiverDebug {
                state: NecState::Init,
                state_new: NecState::Init,
                delta: 0,
                extra: tol.clone(),
            },
            tolerance: tol,
        }
    }

    fn receiver_state(&self) -> ReceiverState<NecCommand> {
        use ReceiverState::*;
        // Internalstate to ReceiverState
        match self.state {
            NecState::Init => Idle,
            NecState::Done => Done(NECTYPE::decode_command(self.bitbuf)),
            NecState::RepeatDone => Done(NECTYPE::decode_command(self.lastcommand)),
            NecState::Err(e) => Error(ReceiverError::Data(0)), //TODO:
            NecState::Disabled => Disabled,
            _ => Receiving,
        }
    }
}

impl<NECTYPE> ReceiverStateMachine for NecTypeReceiver<NECTYPE>
where
    NECTYPE: NecTypeTrait,
{
    type Cmd = NecCommand;
    const ID: ProtocolId = NECTYPE::PROTOCOL;

    fn event(&mut self, rising: bool, time: u32) -> ReceiverState<Self::Cmd> {
        use NecState::*;
        use PulseWidth::*;

        if rising {

            // Calculate the nbr of samples since last rising edge
            let nsamples = time.wrapping_sub(self.last_event);

            // Map the nbr of samples to a pulsewidth
            let pulsewidth = self.tolerance.pulsewidth(nsamples);

            let newstate = match (self.state, pulsewidth) {
                (Init,            Sync)     => Receiving(0),
                (Init,            Repeat)   => RepeatDone,
                (Init,            _)        => Init,

                (Receiving(31),   One)      => {self.bitbuf |= 1 << 31; Done},
                (Receiving(31),   Zero)     => Done,

                (Receiving(bit),  One)      => {self.bitbuf |= 1 << bit; Receiving(bit + 1)},
                (Receiving(bit),  Zero)     => Receiving(bit + 1),

                (Receiving(_),    _)        => Err(ReceiverError::Data(0)),

                (Done,            _)        => Done,
                (RepeatDone,      _)        => RepeatDone,
                (Err(err),        _)        => Err(err),
                (Disabled,        _)        => Disabled,
            };

            #[cfg(feature = "protocol-dev")]
            {
                self.debug.state = self.state;
                self.debug.state_new = newstate;
                self.debug.delta = nsamples as u16;
            }

            self.last_event = time;
            self.state = newstate;
        }

        self.receiver_state()
    }

    fn reset(&mut self) {
        self.state = NecState::Init;
        self.last_event = 0;
        self.lastcommand = if self.bitbuf == 0 {self.lastcommand} else {self.bitbuf};
        self.bitbuf = 0;
    }
}

#[cfg(feature = "embedded-hal")]
mod ehal {
    use super::*;
    use embedded_hal;
    use embedded_hal::digital::v2::InputPin;

    struct HalReceiver<NECTYPE, PIN> {
        sm: NecTypeReceiver<NECTYPE>,
        prev_pinval: bool,
        pin: PIN,
    }

    impl<NECTYPE, PIN> HalReceiver<NECTYPE, PIN>
    where
        NECTYPE: NecTypeTrait,
    {
        pub fn new(pin: PIN, sr: u32) -> Self {
            Self {
                sm: NecTypeReceiver::new(sr),
                prev_pinval: false,
                pin: pin,
            }
        }
    }

    impl<NECTYPE, PIN, PINERR> ReceiverHal<PIN, PINERR, NecCommand> for HalReceiver<NECTYPE, PIN>
    where
        NECTYPE: NecTypeTrait,
        PIN: InputPin<Error=PINERR>,
    {
        fn sample(&mut self, time: u32) -> Result<Option<NecCommand>, PINERR> {

            let pinval = self.pin.is_low()?;

            let rising = pinval && !self.prev_pinval;
            self.prev_pinval = pinval;

            if rising {
                let state = self.sm.event(true, time);

                if let ReceiverState::Done(cmd) = state {
                    return Ok(Some(cmd));
                }

                if ReceiverState::Error == state {
                    self.sm.reset();
                }
            }

            Ok(None)
        }

        fn disable(&mut self) {
            unimplemented!()
        }
    }
}




#[derive(Debug, Clone)]
pub struct Tolerances {
    pub sync: Range<u32>,
    pub repeat: Range<u32>,
    pub zero: Range<u32>,
    pub one: Range<u32>,
}

pub enum PulseWidth {
    Sync,
    Repeat,
    Zero,
    One,
    NotAPulseWidth,
}

impl Tolerances {
    pub const fn from_timing(timing: &Pulsedistance, samplerate: u32) -> Self {
        let per: u32 = 1000 / (samplerate / 1000);
        Tolerances {
            sync: sample_range((timing.header_high + timing.header_low) / per, 5),
            repeat: sample_range((timing.header_high + timing.repeat_low) / per, 5),
            zero: sample_range((timing.data_high + timing.zero_low) / per, 10),
            one: sample_range((timing.data_high + timing.one_low) / per, 10),
        }
    }

    pub fn pulsewidth(&self, samples: u32) -> PulseWidth {
        if self.sync.contains(&samples) {
            return PulseWidth::Sync;
        }
        if self.repeat.contains(&samples) {
            return PulseWidth::Repeat;
        }
        if self.one.contains(&samples) {
            return PulseWidth::One;
        }
        if self.zero.contains(&samples) {
            return PulseWidth::Zero;
        }
        PulseWidth::NotAPulseWidth
    }
}

const fn sample_range(units: u32, percent: u32) -> Range<u32> {
    let tol = (units * percent) / 100;
    (units - tol..units + tol)
}
