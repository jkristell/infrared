use core::ops::Range;

use crate::nec::Pulsedistance;
use crate::{Receiver, ReceiverState, ProtocolId};
#[cfg(feature = "protocol-dev")]
use crate::ReceiverDebug;
use crate::protocols::nec::{NecTypeTrait, NecCommand};
use crate::receiver::ReceiverError;


#[derive(Debug, Clone, Copy)]
/// Error when receiving
pub enum NecError {
    /// Couldn't determine the type of message
    CommandType(u32),
    /// Receiving data but failed to read bit
    Data,
}

pub type NecResult = ReceiverState<NecCommand>;

pub struct NecTypeReceiver<NECTYPE> {
    // State
    state: NecState,
    pub bitbuf: u32,
    prev_sampletime: u32,
    prev_pinval: bool,
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
    Err(NecError),
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
            prev_sampletime: 0,
            prev_pinval: false,
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

    fn receiver_state(&self) -> NecResult {
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

impl<NECTYPE: NecTypeTrait> Receiver for NecTypeReceiver<NECTYPE> {
    type Cmd = NecCommand;
    const PROTOCOL_ID: ProtocolId = NECTYPE::PROTOCOL;

    fn sample(&mut self, pinval: bool, timestamp: u32) -> ReceiverState<NecCommand> {

        let rising_edge = pinval && !self.prev_pinval;
        self.prev_pinval = pinval;

        if rising_edge {
            return self.sample_edge(true, timestamp);
        }

        self.receiver_state()
    }

    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd> {
        use NecState::*;
        use PulseWidth::*;

        if rising {

            let mut delta = sampletime.wrapping_sub(self.prev_sampletime);

            if delta >= core::u16::MAX.into() {
                delta = 0;
            }

            self.prev_sampletime = sampletime;

            let pulsewidth = self.tolerance.pulsewidth(delta);

            let newstate = match (self.state, pulsewidth) {
                (Init,            Sync)     => Receiving(0),
                (Init,            Repeat)   => RepeatDone,
                (Init,            _)        => Init,

                (Receiving(31),   One)      => {self.bitbuf |= 1 << 31; Done},
                (Receiving(31),   Zero)     => Done,

                (Receiving(bit),  One)      => {self.bitbuf |= 1 << bit; Receiving(bit + 1)},
                (Receiving(bit),  Zero)     => Receiving(bit + 1),

                (Receiving(_),    _)        => Err(NecError::Data),

                (Done,            _)        => Done,
                (RepeatDone,      _)        => RepeatDone,
                (Err(err),        _)        => Err(err),
                (Disabled,        _)        => Disabled,
            };

            #[cfg(feature = "protocol-dev")]
            {
                self.debug.state = self.state;
                self.debug.state_new = newstate;
                self.debug.delta = delta as u16;
            }

            self.state = newstate;
        }

        self.receiver_state()
    }

    fn reset(&mut self) {
        self.state = NecState::Init;
        self.prev_sampletime = 0;
        self.prev_pinval = false;
        self.lastcommand = if self.bitbuf == 0 {self.lastcommand} else {self.bitbuf};
        self.bitbuf = 0;
    }

    fn disable(&mut self) {
        self.state = NecState::Disabled;
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
