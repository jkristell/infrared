use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::InputPin;

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
#[cfg(feature = "embedded-hal")]
use crate::receiver::PinReceiver;
use crate::{
    receiver::{time::InfraMonotonic, DecoderBuilder, NoPin, Receiver},
    PeriodicPoll, Protocol,
};

/// Receiver Builder
pub struct Builder<Proto = (), Mono: InfraMonotonic = u32, Cmd = ()> {
    pub(crate) proto: PhantomData<Proto>,
    pub(crate) freq: u32,
    pub(crate) cmd: PhantomData<Cmd>,
    pub(crate) monotonic: PhantomData<Mono>,
}

impl Default for Builder<(), u32, ()> {
    fn default() -> Self {
        Builder {
            proto: PhantomData,
            freq: 1_000_000,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }
}

impl<Proto, Mono, Cmd> Builder<Proto, Mono, Cmd>
where
    Mono: InfraMonotonic,
{
    /// Set the monotonic clock type to use
    pub fn monotonic<NewMono: InfraMonotonic>(self) -> Builder<Proto, NewMono, Cmd> {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    pub fn protocol<NewProto: Protocol>(self) -> Builder<NewProto, Mono, NewProto::Cmd> {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    #[cfg(feature = "nec")]
    pub fn nec(self) -> Builder<Nec, Mono, NecCommand> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec16(self) -> Builder<Nec16, Mono, <Nec16 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_samsung(self) -> Builder<SamsungNec, Mono, <SamsungNec as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    pub fn nec_apple(self) -> Builder<AppleNec, Mono, <AppleNec as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc5")]
    pub fn rc5(self) -> Builder<Rc5, Mono, <Rc5 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc6")]
    pub fn rc6(self) -> Builder<Rc6, Mono, <Rc6 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "sbp")]
    pub fn samsung_bluray(self) -> Builder<Sbp, Mono, <Sbp as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "denon")]
    pub fn denon(self) -> Builder<Denon, Mono, <Denon as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "remotes")]
    /// Use Remote control
    pub fn remotecontrol<Remote>(self, _: Remote) -> Builder<Proto, Mono, Button<Remote>>
    where
        Remote: RemoteControlModel,
    {
        Builder {
            freq: self.freq,
            proto: PhantomData,
            cmd: PhantomData,
            monotonic: PhantomData,
        }
    }

    pub fn frequency(mut self, hz: u32) -> Self {
        self.freq = hz;
        self
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<Proto, Mono, Cmd>
    where
        Proto: DecoderBuilder<Mono>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        Receiver::new(self.freq)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a Receiver with `pin` as input
    pub fn build_with_pin<Pin: InputPin>(
        self,
        pin: Pin,
    ) -> PinReceiver<Receiver<Proto, Mono, Cmd>, Pin>
    where
        Proto: DecoderBuilder<Mono>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PinReceiver::new(Receiver::new(self.freq), pin)
    }

    /// Create a polled Receiver
    pub fn build_polled(self) -> PeriodicPoll<Proto, NoPin, Cmd>
    where
        Proto: DecoderBuilder<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PeriodicPoll::new(self.freq)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a polled Receiver with `pin` as input
    pub fn build_polled_with_pin<Pin: InputPin>(self, pin: Pin) -> PeriodicPoll<Proto, Pin, Cmd>
    where
        Proto: DecoderBuilder<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PeriodicPoll::with_pin(self.freq, pin)
    }
}
