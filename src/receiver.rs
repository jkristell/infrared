//! # Receiver functionality
//!
//! ### Event / Interrupt based Receiver
//!
//! Example:
//! ```
//! use infrared::{Receiver,
//!     remotecontrol::rc5::CdPlayer, cmd::AddressCommand,
//!     protocol::rc5::Rc5Command,
//! };
//! use dummy_pin::DummyPin;
//!
//! // -------------------------------------------
//! // Receiver setup
//! // -------------------------------------------
//!
//! // The pin connected to the receiver
//! let input_pin = DummyPin::new_high();
//!
//! // Resolution of the clock used
//! const RESOLUTION: u32 = 1_000_000;
//!
//! let mut receiver = infrared::receiver()
//!     .rc5()
//!     .frequency(RESOLUTION)
//!     .pin(input_pin)
//!     .remotecontrol(CdPlayer)
//!     .build();
//!
//! // -------------------------------------------
//! // Input interrupt handler
//! // -------------------------------------------
//!
//! let dt = 0; // Time since last pin flip
//!
//! if let Ok(Some(button)) = receiver.event(dt) {
//!     // Get the command associated with this button
//!     let cmd = button.command();
//!     println!(
//!         "Action: {:?} - (Address, Command) = ({}, {})",
//!         button.action(), cmd.address(), cmd.command()
//!     );
//! }
//!
//! ```
//!
//! ### Polled
//!
//! 1. Setup a CountDown-timer at a frequency of something like 20 kHz. How to setup the timer
//! and enable interrupts is HAL-specific but most HALs have examples showing you how to do it.
//!
//! 2. Create a Polled `infrared::Receiver` with the desired Decoder state machine.
//!
//! 3. Periodically call the poll method in the timer interrupt and it should give you a valid command
//! eventually
//!
//! Something like this:
//!
//! #### Polled example
//! ```
//! use embedded_hal::digital::v2::InputPin;
//! use dummy_pin::DummyPin;
//! use infrared::protocol::Nec;
//!
//! // -------------------------------------------
//! // Receiver setup
//! // -------------------------------------------
//!
//! // The pin connected to the receiver hw
//! let input_pin = DummyPin::new_low();
//!
//! // Frequency of the timer interrupt in Hz.
//! const FREQ: u32 = 20_000;
//!
//! let mut receiver = infrared::PeriodicPoll::<Nec, DummyPin>::with_pin(FREQ, input_pin);
//!
//! // -------------------------------------------
//! // Timer interrupt handler
//! // -------------------------------------------
//!
//! if let Ok(Some(cmd)) = receiver.poll() {
//!     println!("{} {}", cmd.addr, cmd.cmd);
//! }
//! ```
//!
//! ## Construction of receiver
//!
//! ```
//!    use infrared::{
//!        Receiver,
//!        receiver::{NoPin, Builder},
//!        protocol::{Rc6, Nec},
//!    };
//!    use dummy_pin::DummyPin;
//!    use infrared::receiver::BufferInputReceiver;
//!
//!    // Receiver for Rc6 signals, event based with embedded-hal pin
//!    let pin = DummyPin::new_low();
//!    let r1: Receiver<Rc6, DummyPin> = Receiver::with_pin(40_000, pin);
//!
//!    // Periodic polled Nec Receiver
//!    let pin = DummyPin::new_low();
//!    let r2: infrared::PeriodicPoll<Nec, DummyPin> = infrared::PeriodicPoll::with_pin(40_000, pin);
//!
//!    let mut r3: BufferInputReceiver<Rc6> = BufferInputReceiver::with_frequenzy(20_000);
//!
//!    let buf: &[u32] = &[20, 40, 20];
//!    let cmd_iter = r3.iter(buf);
//!
//! ```
use core::marker::PhantomData;

#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

use crate::{receiver::time::InfraMonotonic, Protocol};

mod bufferinput;
mod builder;
mod decoder;
mod error;
mod iter;
mod multi;
mod ppoll;
pub mod time;

pub use bufferinput::BufferInputReceiver;
pub use builder::Builder;
pub use decoder::{DecoderBuilder, ProtocolDecoder, State};
pub use error::{DecodingError, Error};
pub use multi::{MultiReceiver};
pub use ppoll::PeriodicPoll;

/// Don't use a embedded-hal pin as input
pub struct NoPin;

/// Event based Receiver
pub struct Receiver<
    Proto: DecoderBuilder<Mono>,
    Pin = NoPin,
    Mono: InfraMonotonic = u32,
    Cmd: From<Proto::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Decoder data
    pub(crate) decoder: Proto::Decoder,
    /// Input
    pub(crate) pin: Pin,
    prev_instant: Mono::Instant,
    /// Type of the final command output
    pub(crate) cmd: PhantomData<Cmd>,
}

impl<Proto, Mono, Cmd> Receiver<Proto, NoPin, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    pub fn new(freq: u32) -> Receiver<Proto, NoPin, Mono, Cmd> {
        let decoder = Proto::build(freq);

        Receiver {
            decoder,
            pin: NoPin {},
            prev_instant: Mono::ZERO_INSTANT,
            cmd: PhantomData,
        }
    }
}

impl<Proto, Input, Mono, Cmd> Receiver<Proto, Input, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    pub fn with_input(freq: u32, input: Input) -> Self {
        let decoder = Proto::build(freq);

        Receiver {
            decoder,
            pin: input,
            prev_instant: Mono::ZERO_INSTANT,
            cmd: PhantomData,
        }
    }

    pub fn event_edge(
        &mut self,
        dt: Mono::Duration,
        edge: bool,
    ) -> Result<Option<Cmd>, DecodingError> {
        // Update state machine
        let state = self.decoder.event(edge, dt);

        match state {
            State::Done => {
                let cmd = self.decoder.command().map(Into::into);
                self.decoder.reset();
                Ok(cmd)
            }
            State::Error(err) => {
                self.decoder.reset();
                Err(err)
            }
            State::Idle | State::Receiving => Ok(None),
        }
    }
}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Mono, Cmd> Receiver<Proto, Pin, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_pin(resolution: u32, pin: Pin) -> Self {
        Self::with_input(resolution, pin)
    }
}

#[cfg(feature = "embedded")]
impl<Proto, Pin, const HZ: u32, Cmd> Receiver<Proto, Pin, fugit::TimerInstantU32<HZ>, Cmd>
where
    Proto: DecoderBuilder<fugit::TimerInstantU32<HZ>>,
    Pin: InputPin,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_fugit(pin: Pin) -> Self {
        Self::with_input(HZ, pin)
    }
}

#[cfg(feature = "embedded")]
impl<Proto, Pin, const HZ: u32, Cmd> Receiver<Proto, Pin, fugit::TimerInstantU64<HZ>, Cmd>
where
    Proto: DecoderBuilder<fugit::TimerInstantU64<HZ>>,
    Pin: InputPin,
    Cmd: From<Proto::Cmd>,
{
    /// Create a `Receiver` with `pin` as input
    pub fn with_fugit64(pin: Pin) -> Self {
        Self::with_input(HZ, pin)
    }
}

impl<Proto, Mono, Cmd> Receiver<Proto, NoPin, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    pub fn event(&mut self, dt: Mono::Duration, edge: bool) -> Result<Option<Cmd>, DecodingError> {
        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    pub fn event_instant(
        &mut self,
        t: Mono::Instant,
        edge: bool,
    ) -> Result<Option<Cmd>, DecodingError> {
        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }
}

#[cfg(feature = "embedded-hal")]
impl<Proto, Pin, Mono, Cmd> Receiver<Proto, Pin, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Pin: InputPin,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    pub fn event(&mut self, dt: Mono::Duration) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.pin.is_low().map_err(Error::Hal)?;
        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    pub fn event_instant(&mut self, t: Mono::Instant) -> Result<Option<Cmd>, Error<Pin::Error>> {
        let edge = self.pin.is_low().map_err(Error::Hal)?;

        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        Ok(self.event_edge(dt, edge)?.map(Into::into))
    }

    /// Get a reference to the Pin
    pub fn pin(&self) -> &Pin {
        &self.pin
    }

    /// Get a mut ref to the Pin
    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.pin
    }

    /// Drop the receiver and release the pin
    pub fn release(self) -> Pin {
        self.pin
    }
}
