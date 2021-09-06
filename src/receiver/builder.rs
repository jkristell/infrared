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
    pub(crate) resolution: usize,
    pub(crate) from_cmd: PhantomData<C>,
}

impl Builder<DummyProtocol, Event, DefaultInput, ()> {
    pub fn new() -> Builder<DummyProtocol, Event, DefaultInput> {
        Builder {
            proto: PhantomData::default(),
            input: DefaultInput {},
            method: Default::default(),
            resolution: 1_000_000,
            from_cmd: Default::default(),
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
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
            from_cmd: Default::default(),
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

    #[cfg(feature = "remotes")]
    pub fn remote<Remote>(self) -> Builder<SM, S, IN, Button<Remote>>
    where
        Remote: RemoteControlModel,
    {
        Builder {
            resolution: self.resolution,
            proto: PhantomData::default(),
            input: self.input,
            method: self.method,
            from_cmd: PhantomData::default(),
        }
    }

    #[cfg(feature = "embedded-hal")]
    /// The Receiver use `pin` as input
    pub fn pin<PIN: InputPin>(self, pin: PIN) -> Builder<SM, S, PinInput<PIN>, C> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: PinInput(pin),
            method: Default::default(),
            from_cmd: self.from_cmd,
        }
    }

    /// The Receiver should read the data from a data buffer
    pub fn buffer(self, buf: &[usize]) -> Builder<SM, Event, BufferInput<'_>, C> {
        Builder {
            proto: self.proto,
            input: BufferInput(buf),
            method: Default::default(),
            resolution: self.resolution,
            from_cmd: self.from_cmd,
        }
    }

    pub fn resolution(mut self, hz: usize) -> Self {
        self.resolution = hz;
        self
    }

    /// Periodic Polled
    pub fn polled(self) -> Builder<SM, Poll, IN, C> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: self.input,
            method: Default::default(),
            from_cmd: self.from_cmd,
        }
    }

    /// Event driven
    pub fn event_driven(self) -> Builder<SM, Event, IN, C> {
        Builder {
            resolution: self.resolution,
            proto: self.proto,
            input: self.input,
            method: Default::default(),
            from_cmd: self.from_cmd,
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
    pub fn build(self) -> Receiver<SM, S, IN, C>
    where
        SM: DecoderStateMachine,
    {
        Receiver::with_input(self.resolution, self.input)
    }
}
