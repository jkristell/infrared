use crate::receiver::{
    BufferInput, ConstDecodeStateMachine, ConstReceiver, DecoderStateMachine, DefaultInput, Event,
    PinInput, Poll, Receiver,
};
use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

#[cfg(feature = "nec")]
use crate::protocol::{Nec, Nec16, NecApple, NecSamsung};

#[cfg(feature = "denon")]
use crate::protocol::Denon;
#[cfg(feature = "rc5")]
use crate::protocol::Rc5;
#[cfg(feature = "rc6")]
use crate::protocol::Rc6;
#[cfg(feature = "sbp")]
use crate::protocol::Sbp;

/// Receiver Builder
pub struct Builder<SM, S = Event, IN = DefaultInput> {
    pub(crate) proto: PhantomData<SM>,
    pub(crate) input: IN,
    pub(crate) method: PhantomData<S>,
    pub(crate) resolution: usize,
}

impl<SM, S, IN> Builder<SM, S, IN>
where
    S: Default,
    SM: DecoderStateMachine,
{
    pub fn new() -> Builder<SM, Event, DefaultInput> {
        Builder {
            proto: PhantomData::default(),
            input: DefaultInput {},
            method: Default::default(),
            resolution: 1_000_000,
        }
    }

    pub fn protocol<Proto>(self) -> Builder<Proto, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec(self) -> Builder<Nec, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec16(self) -> Builder<Nec16, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec_samsung(self) -> Builder<NecSamsung, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec_apple(self) -> Builder<NecApple, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "rc5")]
    pub fn rc5(self) -> Builder<Rc5, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "rc6")]
    pub fn rc6(self) -> Builder<Rc6, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "sbp")]
    pub fn samsung_bluray(self) -> Builder<Sbp, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "denon")]
    pub fn denon(self) -> Builder<Denon, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
        }
    }

    #[cfg(feature = "embedded-hal")]
    /// The Receiver use `pin` as input
    pub fn pin<PIN: InputPin>(self, pin: PIN) -> Builder<SM, S, PinInput<PIN>> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: PinInput(pin),
            method: Default::default(),
        }
    }

    /// The Receiver should read the data from a data buffer
    pub fn buffer(self, buf: &[usize]) -> Builder<SM, Event, BufferInput<'_>> {
        Builder {
            proto: self.proto,
            input: BufferInput(buf),
            method: Default::default(),
            resolution: self.resolution,
        }
    }

    pub fn resolution(mut self, hz: usize) -> Self {
        self.resolution = hz;
        self
    }

    /// Periodic Polled
    pub fn polled(self) -> Builder<SM, Poll, IN> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: self.input,
            method: Default::default(),
        }
    }

    /// Event driven
    pub fn event_driven(self) -> Builder<SM, Event, IN> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: self.input,
            method: Default::default(),
        }
    }

    /// Create a Receiver with resolution known at build time
    pub fn build_const<const R: usize>(self) -> ConstReceiver<SM, S, IN, R>
    where
        SM: ConstDecodeStateMachine<R>,
    {
        ConstReceiver::with_input(self.input)
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<SM, S, IN>
    where
        SM: DecoderStateMachine,
        S: Default,
    {
        Receiver::new(self.resolution, self.input)
    }
}
