use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

#[cfg(feature = "denon")]
use crate::protocol::Denon;
#[cfg(feature = "rc5")]
use crate::protocol::Rc5;
#[cfg(feature = "rc6")]
use crate::protocol::Rc6;
#[cfg(feature = "sbp")]
use crate::protocol::Sbp;
#[cfg(feature = "nec")]
use crate::protocol::{nec::NecCommand, AppleNec, Nec, Nec16, SamsungNec};
#[cfg(feature = "remotes")]
use crate::remotecontrol::{Button, RemoteControlModel};
use crate::{
    receiver::{time::InfraMonotonic, DecoderBuilder, NoPin, Receiver},
    PeriodicPoll, Protocol,
};

/// Receiver Builder
pub struct Builder<Proto = (), Pin = NoPin, Mono: InfraMonotonic = u32, Cmd = ()> {
    pub(crate) proto: PhantomData<Proto>,
    pub(crate) pin: Pin,
    pub(crate) freq: u32,
    pub(crate) cmd: PhantomData<Cmd>,
    pub(crate) monotonic: PhantomData<Mono>,
}

impl Default for Builder<(), NoPin, u32, ()> {
    fn default() -> Self {
        Builder {
            proto: PhantomData,
            pin: NoPin,
            freq: 1_000_000,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }
}

impl<Proto, Input, Mono, Cmd> Builder<Proto, Input, Mono, Cmd>
where
    Mono: InfraMonotonic,
{
    /// Set the monotonic clock type to use
    pub fn monotonic<NewMono: InfraMonotonic>(self) -> Builder<Proto, Input, NewMono, Cmd> {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            pin: self.pin,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    pub fn protocol<NewProto: Protocol>(self) -> Builder<NewProto, Input, Mono, NewProto::Cmd> {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            pin: self.pin,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec(self) -> Builder<Nec, Input, Mono, NecCommand> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec16(self) -> Builder<Nec16, Input, Mono, <Nec16 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_samsung(self) -> Builder<SamsungNec, Input, Mono, <SamsungNec as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_apple(self) -> Builder<AppleNec, Input, Mono, <AppleNec as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc5")]
    pub fn rc5(self) -> Builder<Rc5, Input, Mono, <Rc5 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc6")]
    pub fn rc6(self) -> Builder<Rc6, Input, Mono, <Rc6 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "sbp")]
    pub fn samsung_bluray(self) -> Builder<Sbp, Input, Mono, <Sbp as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "denon")]
    pub fn denon(self) -> Builder<Denon, Input, Mono, <Denon as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "remotes")]
    /// Use Remote control
    pub fn remotecontrol<Remote>(self, _: Remote) -> Builder<Proto, Input, Mono, Button<Remote>>
    where
        Remote: RemoteControlModel,
    {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            pin: self.pin,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    #[cfg(feature = "embedded-hal")]
    /// The Receiver use `pin` as input
    pub fn pin<NewPin: InputPin>(self, pin: NewPin) -> Builder<Proto, NewPin, Mono, Cmd> {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            pin,
            monotonic: PhantomData,
            cmd: PhantomData,
        }
    }

    pub fn frequency(mut self, hz: u32) -> Self {
        self.freq = hz;
        self
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<Proto, Input, Mono, Cmd>
    where
        Proto: DecoderBuilder<Mono>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        Receiver::with_input(self.freq, self.pin)
    }

    pub fn build_polled(self) -> PeriodicPoll<Proto, Input, Cmd>
    where
        Proto: DecoderBuilder<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PeriodicPoll::with_input(self.freq, self.pin)
    }
}
