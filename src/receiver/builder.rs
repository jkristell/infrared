use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::InputPin;

#[cfg(feature = "denon")]
use crate::protocol::Denon;
#[cfg(feature = "rc5")]
use crate::protocol::Rc5;
#[cfg(feature = "rc6")]
use crate::protocol::Rc6;
#[cfg(feature = "samsung36")]
use crate::protocol::Samsung36;
#[cfg(feature = "nec")]
use crate::protocol::{nec::NecCommand, AppleNec, Nec, NecExtended, Samsung32};
#[cfg(feature = "async")]
use crate::receiver::AsyncReceiver;
#[cfg(feature = "embassy")]
use crate::receiver::EmbassyClock;
#[cfg(feature = "embedded-hal")]
use crate::receiver::PinReceiver;
#[cfg(feature = "remotes")]
use crate::remotecontrol::{Button, RemoteControl};
use crate::{
    receiver::{time::TimeBase, Clock, ClockedReceiver, DecoderBuilder, Receiver},
    PolledReceiver, Protocol,
};

#[cfg(feature = "embedded-hal")]
type ClockedPinReceiver<Proto, C, Cmd, Pin> =
    PinReceiver<ClockedReceiver<Receiver<Proto, <C as Clock>::Time, Cmd>, C>, Pin>;

/// Receiver Builder
pub struct Builder<Proto = (), Time: TimeBase = u32, Cmd = ()> {
    pub(crate) proto: PhantomData<Proto>,
    pub(crate) resolution: u32,
    pub(crate) cmd: PhantomData<Cmd>,
    pub(crate) time: PhantomData<Time>,
}

impl Default for Builder<(), u32, ()> {
    fn default() -> Self {
        Builder {
            proto: PhantomData,
            resolution: 1_000_000,
            cmd: PhantomData,
            time: PhantomData,
        }
    }
}

impl<Proto, Time, Cmd> Builder<Proto, Time, Cmd>
where
    Time: TimeBase,
{
    /// Set the [`TimeBase`] used for timestamps and durations, e.g. `u64` or a fugit instant
    pub fn timebase<NewTime: TimeBase>(self) -> Builder<Proto, NewTime, Cmd> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            cmd: PhantomData,
            time: PhantomData,
        }
    }

    /// Use protocol `NewProto`
    pub fn protocol<NewProto: Protocol>(self) -> Builder<NewProto, Time, NewProto::Cmd> {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            cmd: PhantomData,
            time: PhantomData,
        }
    }

    #[cfg(feature = "nec")]
    /// Use the NEC protocol
    pub fn nec(self) -> Builder<Nec, Time, NecCommand> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    /// Use the NEC extended protocol, NEC with 16 bit addresses
    pub fn nec_extended(self) -> Builder<NecExtended, Time, <NecExtended as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    /// Use the Samsung32 protocol, the NEC variant used by Samsung TVs
    pub fn samsung32(self) -> Builder<Samsung32, Time, <Samsung32 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "nec")]
    /// Use the Apple NEC variant
    pub fn apple_nec(self) -> Builder<AppleNec, Time, <AppleNec as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc5")]
    /// Use the Philips RC5 protocol
    pub fn rc5(self) -> Builder<Rc5, Time, <Rc5 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "rc6")]
    /// Use the Philips RC6 (mode 0) protocol
    pub fn rc6(self) -> Builder<Rc6, Time, <Rc6 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "samsung36")]
    /// Use the Samsung36 protocol, used by Samsung Blu-ray players
    pub fn samsung36(self) -> Builder<Samsung36, Time, <Samsung36 as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "denon")]
    /// Use the Denon-K protocol
    pub fn denon(self) -> Builder<Denon, Time, <Denon as Protocol>::Cmd> {
        self.protocol()
    }

    #[cfg(feature = "remotes")]
    /// Use Remote control
    pub fn remotecontrol<Remote>(self, _: Remote) -> Builder<Proto, Time, Button<Remote>>
    where
        Remote: RemoteControl,
    {
        Builder {
            resolution: self.resolution,
            proto: PhantomData,
            cmd: PhantomData,
            time: PhantomData,
        }
    }

    /// Set the resolution of the timer, in Hz
    pub fn resolution(mut self, hz: u32) -> Self {
        self.resolution = hz;
        self
    }

    /// Create the Receiver
    pub fn build(self) -> Receiver<Proto, Time, Cmd>
    where
        Proto: DecoderBuilder<Time>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        Receiver::new(self.resolution)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a Receiver with `pin` as input
    pub fn build_with_pin<Pin: InputPin>(
        self,
        pin: Pin,
    ) -> PinReceiver<Receiver<Proto, Time, Cmd>, Pin>
    where
        Proto: DecoderBuilder<Time>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PinReceiver::new(Receiver::new(self.resolution), pin)
    }

    /// Create a Receiver that timestamps the edges with `clock`
    ///
    /// The resolution and time representation are taken from the clock.
    pub fn build_clocked<C>(self, clock: C) -> ClockedReceiver<Receiver<Proto, C::Time, Cmd>, C>
    where
        C: Clock,
        Proto: DecoderBuilder<C::Time>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        ClockedReceiver::with_clock(clock)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a Receiver that timestamps the edges with `clock`, with `pin` as input
    ///
    /// The resolution and time representation are taken from the clock.
    pub fn build_clocked_with_pin<C, Pin: InputPin>(
        self,
        clock: C,
        pin: Pin,
    ) -> ClockedPinReceiver<Proto, C, Cmd, Pin>
    where
        C: Clock,
        Proto: DecoderBuilder<C::Time>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PinReceiver::new(ClockedReceiver::with_clock(clock), pin)
    }

    #[cfg(feature = "async")]
    /// Create an async Receiver with `pin` as input, `clock` for timestamping
    /// the edges and `delay` for the idle timeout
    ///
    /// The resolution and time representation are taken from the clock.
    pub fn build_async<C, Pin, D>(
        self,
        clock: C,
        pin: Pin,
        delay: D,
    ) -> AsyncReceiver<Receiver<Proto, C::Time, Cmd>, C, Pin, D>
    where
        C: Clock,
        Proto: DecoderBuilder<C::Time>,
        Cmd: From<<Proto as Protocol>::Cmd>,
        Pin: embedded_hal_async::digital::Wait,
        D: embedded_hal_async::delay::DelayNs,
    {
        AsyncReceiver::with_clock(clock, pin, delay)
    }

    #[cfg(feature = "embassy")]
    /// Create an async Receiver with `pin` as input, using embassy-time
    pub fn build_embassy<Pin>(
        self,
        pin: Pin,
    ) -> AsyncReceiver<Receiver<Proto, u64, Cmd>, EmbassyClock, Pin, embassy_time::Delay>
    where
        Proto: DecoderBuilder<u64>,
        Cmd: From<<Proto as Protocol>::Cmd>,
        Pin: embedded_hal_async::digital::Wait,
    {
        AsyncReceiver::embassy(pin)
    }

    /// Create a polled Receiver
    pub fn build_polled(self) -> PolledReceiver<Proto, Cmd>
    where
        Proto: DecoderBuilder<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PolledReceiver::new(self.resolution)
    }

    #[cfg(feature = "embedded-hal")]
    /// Create a polled Receiver with `pin` as input
    pub fn build_polled_with_pin<Pin: InputPin>(
        self,
        pin: Pin,
    ) -> PinReceiver<PolledReceiver<Proto, Cmd>, Pin>
    where
        Proto: DecoderBuilder<u32>,
        Cmd: From<<Proto as Protocol>::Cmd>,
    {
        PinReceiver::new(PolledReceiver::new(self.resolution), pin)
    }
}
