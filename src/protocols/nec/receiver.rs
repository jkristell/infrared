
use crate::nec::NecTiming;
use crate::{ReceiverStateMachine, ReceiverState, ProtocolId};
#[cfg(feature = "protocol-dev")]
use crate::ReceiverDebug;
use crate::protocols::nec::{NecTypeTrait, NecCommand};
use crate::receiver::{ReceiverError};
use crate::protocols::utils::{Ranges};


pub struct NecTypeReceiver<NECTYPE> {
    // State
    state: NecState,
    // Time of last event
    last_event: u32,
    // Data buffer
    pub bitbuf: u32,
    // Timing and tolerances
    ranges: Ranges<PulseWidth>,
    // Last command (used by repeat)
    lastcommand: u32,
    // The type of Nec
    nectype: core::marker::PhantomData<NECTYPE>,

    #[cfg(feature = "protocol-dev")]
    pub debug: ReceiverDebug<NecState, Ranges<PulseWidth>>,
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
        let timing = NECTYPE::PULSEDISTANCE;
        Self::new_from_timing(samplerate, timing)
    }

    fn new_from_timing(samplerate: u32, timing: &NecTiming) -> Self {
        let nsamples = nsamples_from_timing(timing, samplerate);
        let ranges = Ranges::new(&nsamples);

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
            ranges,
        }
    }

    fn receiver_state(&self) -> ReceiverState<NecCommand> {
        use NecState::*;
        // Internalstate to ReceiverState
        match self.state {
            Init        => ReceiverState::Idle,
            Done        => ReceiverState::Done(NECTYPE::decode_command(self.bitbuf)),
            RepeatDone  => ReceiverState::Done(NECTYPE::decode_command(self.lastcommand)),
            Err(e) => ReceiverState::Error(e),
            Disabled    => ReceiverState::Disabled,
            _           => ReceiverState::Receiving,
        }
    }

    #[cfg(feature = "protocol-dev")]
    fn update_debug(&mut self, newstate: NecState, nsamples: u32) {
        self.debug.state = self.state;
        self.debug.state_new = newstate;
        self.debug.delta = nsamples as u16;
    }
}

impl<NECTYPE> ReceiverStateMachine for NecTypeReceiver<NECTYPE>
where
    NECTYPE: NecTypeTrait,
{
    type Cmd = NecCommand;
    const ID: ProtocolId = NECTYPE::PROTOCOL;

    fn for_samplerate(samplerate: u32) -> Self {
        let timing = NECTYPE::PULSEDISTANCE;
        Self::new_from_timing(samplerate, timing)
    }


    fn event(&mut self, rising: bool, time: u32) -> ReceiverState<Self::Cmd> {
        use NecState::*;
        use PulseWidth::*;

        if rising {
            // Calculate the nbr of samples since last rising edge
            let nsamples = time.wrapping_sub(self.last_event);

            // Map the nbr of samples to a pulsewidth
            let pulsewidth = self.ranges.pulsewidth(nsamples);

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
            self.update_debug(newstate, nsamples);

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


#[derive(Debug)]
enum PulseWidth {
    Sync = 0,
    Repeat = 1,
    Zero = 2,
    One = 3,
    NotAPulseWidth = 4,
}


impl Default for PulseWidth {
    fn default() -> Self {
        PulseWidth::NotAPulseWidth
    }
}

impl From<usize> for PulseWidth {
    fn from(v: usize) -> Self {
        match v {
            0 => PulseWidth::Sync,
            1 => PulseWidth::Repeat,
            2 => PulseWidth::Zero,
            3 => PulseWidth::One,
            _ => PulseWidth::NotAPulseWidth,
        }
    }
}


const fn nsamples_from_timing(t: &NecTiming, samplerate: u32) -> [(u32, u32); 4] {
    let per: u32 = 1000 / (samplerate / 1000);
    [
        ((t.hh + t.hl) / per, 5),
        ((t.hh + t.rl) / per, 5),
        ((t.dh + t.zl) / per, 10),
        ((t.dh + t.ol) / per, 10),
    ]
}




