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

use crate::{
    protocol::DummyProtocol,
    remotecontrol::{Button, RemoteControlModel},
    Protocol,
};

/// Receiver Builder
pub struct Builder<
    SM: Protocol = DummyProtocol,
    S = Event,
    IN = DefaultInput,
    C = <SM as Protocol>::Cmd,
> {
    pub(crate) proto: PhantomData<SM>,
    pub(crate) input: IN,
    pub(crate) method: PhantomData<S>,
    pub(crate) resolution: u32,
    pub(crate) output: PhantomData<C>,
}

impl Builder<DummyProtocol, Event, DefaultInput, ()> {
    pub fn new() -> Builder<DummyProtocol, Event, DefaultInput> {
        Builder {
            proto: PhantomData,
            input: DefaultInput {},
            method: PhantomData,
            resolution: 1_000_000,
            output: PhantomData,
        }
    }
}

impl<SM, S, IN, C> Builder<SM, S, IN, C>
where
    S: Default,
    SM: Protocol,
    C: From<<SM as Protocol>::Cmd>,
{
    pub fn protocol<Proto: Protocol>(self) -> Builder<Proto, S, IN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            method: PhantomData,
            output: PhantomData,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec(self) -> Builder<Nec, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec16(self) -> Builder<Nec16, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_samsung(self) -> Builder<NecSamsung, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_apple(self) -> Builder<NecApple, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "rc5")]
    pub fn rc5(self) -> Builder<Rc5, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "rc6")]
    pub fn rc6(self) -> Builder<Rc6, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "sbp")]
    pub fn samsung_bluray(self) -> Builder<Sbp, S, IN> {
        self.protocol()
    }

    #[cfg(feature = "denon")]
    pub fn denon(self) -> Builder<Denon, S, IN> {
        self.protocol()
    }

    /// Use Remote control
    pub fn remotecontrol<Remote>(self, _: Remote) -> Builder<SM, S, IN, Button<Remote>>
    where
        Remote: RemoteControlModel,
    {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            method: PhantomData,
            output: PhantomData,
        }
    }

    #[cfg(feature = "embedded-hal")]
    /// The Receiver use `pin` as input
    pub fn pin<PIN: InputPin>(self, pin: PIN) -> Builder<SM, S, PinInput<PIN>, C> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: PinInput(pin),
            method: PhantomData,
            output: PhantomData,
        }
    }

    /// The Receiver should read the data from a data buffer
    pub fn buffer(self, buf: &[u32]) -> Builder<SM, Event, BufferInput<'_>, C> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: BufferInput(buf),
            method: PhantomData,
            output: PhantomData,
        }
    }

    pub fn resolution(mut self, hz: u32) -> Self {
        self.resolution = hz;
        self
    }

    /// Periodic Polled
    pub fn polled(self) -> Builder<SM, Poll, IN, C> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            method: PhantomData,
            output: PhantomData,
        }
    }

    /// Event driven
    pub fn event_driven(self) -> Builder<SM, Event, IN, C> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            method: PhantomData,
            output: PhantomData,
        }
    }

    /// Create a Receiver with resolution known at build time
    pub fn build_const<const R: u32>(self) -> ConstReceiver<SM, S, IN, R>
    where
        SM: ConstDecodeStateMachine<R>,
    {
        ConstReceiver::with_input(self.input)
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<SM, S, IN, C>
    where
        SM: DecoderStateMachine,
    {
        Receiver::with_input(self.resolution, self.input)
    }
}
