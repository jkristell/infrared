use crate::receiver::{ProtocolDecoder, NoPinInput, Receiver, ProtocolDecoderAdaptor};
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

use crate::{protocol::DummyProtocol, Protocol};

use crate::receiver::time::InfraMonotonic;
#[cfg(feature = "remotes")]
use crate::remotecontrol::{Button, RemoteControlModel};

/// Receiver Builder
pub struct Builder<
    Proto: Protocol = DummyProtocol,
    IN = NoPinInput,
    T: InfraMonotonic = u32,
    C = <Proto as Protocol>::Cmd,
> {
    pub(crate) proto: PhantomData<Proto>,
    pub(crate) input: IN,
    pub(crate) resolution: u32,
    pub(crate) output: PhantomData<C>,
    pub(crate) monotonic: PhantomData<T>,
}

impl Builder<DummyProtocol, NoPinInput, u32, ()> {
    pub fn new() -> Builder<DummyProtocol, NoPinInput> {
        Builder {
            proto: PhantomData,
            input: NoPinInput {},
            resolution: 1_000_000,
            output: PhantomData,
            monotonic: PhantomData,
        }
    }
}

impl<Proto, Input, Mono, C> Builder<Proto, Input, Mono, C>
where
    Proto: Protocol,
    Mono: InfraMonotonic,
    C: From<<Proto as Protocol>::Cmd>,
{
    /// Set the monotonic clock type to use
    pub fn monotonic<NewMono: InfraMonotonic>(self) -> Builder<Proto, Input, NewMono> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            output: PhantomData,
            monotonic: PhantomData,
        }
    }

    pub fn protocol<NewProto: Protocol>(self) -> Builder<NewProto, Input, Mono> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            output: PhantomData,
            monotonic: PhantomData,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec(self) -> Builder<Nec, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec16(self) -> Builder<Nec16, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_samsung(self) -> Builder<NecSamsung, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_apple(self) -> Builder<NecApple, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "rc5")]
    pub fn rc5(self) -> Builder<Rc5, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "rc6")]
    pub fn rc6(self) -> Builder<Rc6, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "sbp")]
    pub fn samsung_bluray(self) -> Builder<Sbp, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "denon")]
    pub fn denon(self) -> Builder<Denon, Input, Mono> {
        self.protocol()
    }

    #[cfg(feature = "remotes")]
    /// Use Remote control
    pub fn remotecontrol<Remote>(self, _: Remote) -> Builder<Proto, Input, Mono, Button<Remote>>
    where
        Remote: RemoteControlModel,
    {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: self.input,
            output: PhantomData,
            monotonic: PhantomData,
        }
    }

    #[cfg(feature = "embedded-hal")]
    /// The Receiver use `pin` as input
    pub fn pin<PIN: InputPin>(self, pin: PIN) -> Builder<Proto, PIN> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            input: pin,
            monotonic: PhantomData,
            output: PhantomData,
        }
    }

    pub fn resolution(mut self, hz: u32) -> Self {
        self.resolution = hz;
        self
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<Proto, Input, Mono, C>
    where
        Proto: ProtocolDecoderAdaptor<Mono>,
    {
        Receiver::with_input(self.resolution, self.input)
    }
}
