//! # Receiver functionality
//!
//! The receivers are split into a hardware independent core and wrappers
//! that handle the input:
//!
//!  - [`Receiver`]: The core, protocol decoder state machine. Edge events are
//!    fed to it manually. Useful on its own in interrupt handlers where the
//!    pin is managed by the application, in host applications, and as the
//!    building block for the wrappers.
//!  - [`PinReceiver`]: [`Receiver`] combined with an embedded-hal `InputPin`.
//!  - [`MultiReceiver`]: Multiple protocol decoders sharing the input signal.
//!  - [`PeriodicPoll`]: Polling based receiver.
//!  - [`BufferReceiver`]: Decodes buffers of edge timings.
//!
//! ### Event / Interrupt based Receiver with embedded-hal pin
//!
//! Example:
//! ```
//! use infrared::{PinReceiver,
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
//!     .resolution(RESOLUTION)
//!     .remotecontrol(CdPlayer)
//!     .build_with_pin(input_pin);
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
//! ### Event based Receiver without a pin
//!
//! The core [`Receiver`] is hardware independent and can be used by feeding
//! it edge events manually:
//! ```
//! use infrared::{Receiver, protocol::Nec};
//!
//! // Resolution of the clock used
//! const RESOLUTION: u32 = 1_000_000;
//!
//! let mut receiver: Receiver<Nec> = Receiver::new(RESOLUTION);
//!
//! // -------------------------------------------
//! // Input interrupt handler
//! // -------------------------------------------
//!
//! let dt = 0;        // Time since last pin flip
//! let edge = false;  // Edge read from the pin
//!
//! if let Ok(Some(cmd)) = receiver.event(dt, edge) {
//!     println!("{} {}", cmd.addr, cmd.cmd);
//! }
//! ```
//!
//! ### Polled
//!
//! 1. Setup a CountDown-timer at a frequency of something like 20 kHz. How to setup the timer
//!    and enable interrupts is HAL-specific but most HALs have examples showing you how to do it.
//!
//! 2. Create a Polled `infrared::PeriodicPoll` with the desired Decoder state machine.
//!
//! 3. Periodically call the poll method in the timer interrupt and it should give you a valid command
//!    eventually
//!
//! Something like this:
//!
//! #### Polled example
//! ```
//! use embedded_hal::digital::InputPin;
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
//! let mut receiver = infrared::PeriodicPoll::<Nec>::with_pin(FREQ, input_pin);
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
//!        Receiver, PinReceiver,
//!        receiver::Builder,
//!        protocol::{Rc6, Nec},
//!    };
//!    use dummy_pin::DummyPin;
//!    use infrared::receiver::BufferReceiver;
//!
//!    // Receiver for Rc6 signals, event based with embedded-hal pin
//!    let pin = DummyPin::new_low();
//!    let r1: PinReceiver<Receiver<Rc6>, DummyPin> = PinReceiver::with_pin(40_000, pin);
//!
//!    // Periodic polled Nec Receiver
//!    let pin = DummyPin::new_low();
//!    let r2: PinReceiver<infrared::PeriodicPoll<Nec>, DummyPin> =
//!        infrared::PeriodicPoll::with_pin(40_000, pin);
//!
//!    let r3: BufferReceiver<Rc6> = BufferReceiver::with_resolution(20_000);
//!
//!    let buf: &[u32] = &[20, 40, 20];
//!    let cmd_iter = r3.iter(buf);
//!
//! ```
use core::marker::PhantomData;

use crate::{receiver::time::InfraMonotonic, Protocol};

mod bufferinput;
mod builder;
mod decoder;
mod error;
mod iter;
mod multi;
#[cfg(feature = "embedded-hal")]
mod pin;
mod ppoll;
pub mod time;

pub use bufferinput::BufferReceiver;
pub use iter::BufferIterator;
pub use builder::Builder;
pub use decoder::{DecoderBuilder, ProtocolDecoder, State};
pub use error::{DecodingError, Error};
pub use multi::{MultiReceiver, ReceiverWrapper};
#[cfg(feature = "embedded-hal")]
pub use pin::PinReceiver;
pub use ppoll::PeriodicPoll;

/// Edge driven receiver
///
/// Implemented by the pinless receiver cores ([`Receiver`] and
/// [`MultiReceiver`]) so that input wrappers such as [`PinReceiver`] can be
/// generic over them.
pub trait EdgeReceiver {
    /// Duration type of the monotonic clock
    type Duration;
    /// Instant type of the monotonic clock
    type Instant;
    /// Output produced on a successful decode
    type Output;

    /// Update the receiver with an edge event
    /// * `dt`: Duration since the previous edge
    /// * `edge`: true = start of an IR mark (the pin reading low on an active-low receiver module), false = start of a space
    fn event_edge(&mut self, dt: Self::Duration, edge: bool)
        -> Result<Self::Output, DecodingError>;

    /// Update the receiver with an edge event at instant `t`
    fn event_edge_instant(
        &mut self,
        t: Self::Instant,
        edge: bool,
    ) -> Result<Self::Output, DecodingError>;
}

/// Poll driven receiver
///
/// Implemented by [`PeriodicPoll`] so that input wrappers such as
/// [`PinReceiver`] can be generic over it.
pub trait PollReceiver {
    /// Output produced on a successful decode
    type Output;

    /// Update the receiver with the current pin state
    fn poll_edge(&mut self, edge: bool) -> Result<Self::Output, DecodingError>;
}

/// Event based Receiver
///
/// The core protocol decoder state machine. Hardware independent: edge events
/// are fed to it via [`event`](Receiver::event) or
/// [`event_instant`](Receiver::event_instant). Combine it with
/// [`PinReceiver`] to let it read the edges from an embedded-hal pin.
pub struct Receiver<
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic = u32,
    Cmd: From<Proto::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Decoder data
    pub(crate) decoder: Proto::Decoder,
    /// Previous instant, used by `event_instant`
    prev_instant: Mono::Instant,
    /// Type of the final command output
    pub(crate) cmd: PhantomData<Cmd>,
}

impl<Proto, Mono, Cmd> Receiver<Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    /// Create a new Receiver with the given clock resolution (in Hz)
    pub fn new(freq: u32) -> Self {
        Receiver {
            decoder: Proto::build(freq),
            prev_instant: Mono::ZERO_INSTANT,
            cmd: PhantomData,
        }
    }

    /// Update the receiver
    /// * `dt`: Duration since the previous edge
    /// * `edge`: true = start of an IR mark (the pin reading low on an active-low receiver module), false = start of a space
    pub fn event(&mut self, dt: Mono::Duration, edge: bool) -> Result<Option<Cmd>, DecodingError> {
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

    /// Update the receiver with an edge event at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_instant(
        &mut self,
        t: Mono::Instant,
        edge: bool,
    ) -> Result<Option<Cmd>, DecodingError> {
        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        self.event(dt, edge)
    }
}

#[cfg(feature = "fugit")]
impl<Proto, const HZ: u32, Cmd> Receiver<Proto, fugit::TimerInstantU32<HZ>, Cmd>
where
    Proto: DecoderBuilder<fugit::TimerInstantU32<HZ>>,
    Cmd: From<Proto::Cmd>,
{
    /// Create a new Receiver with fugit `TimerInstantU32<HZ>` as its clock
    pub fn with_fugit() -> Self {
        Self::new(HZ)
    }
}

#[cfg(feature = "fugit")]
impl<Proto, const HZ: u32, Cmd> Receiver<Proto, fugit::TimerInstantU64<HZ>, Cmd>
where
    Proto: DecoderBuilder<fugit::TimerInstantU64<HZ>>,
    Cmd: From<Proto::Cmd>,
{
    /// Create a new Receiver with fugit `TimerInstantU64<HZ>` as its clock
    pub fn with_fugit64() -> Self {
        Self::new(HZ)
    }
}

impl<Proto, Mono, Cmd> EdgeReceiver for Receiver<Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<Proto::Cmd>,
{
    type Duration = Mono::Duration;
    type Instant = Mono::Instant;
    type Output = Option<Cmd>;

    fn event_edge(
        &mut self,
        dt: Self::Duration,
        edge: bool,
    ) -> Result<Self::Output, DecodingError> {
        self.event(dt, edge)
    }

    fn event_edge_instant(
        &mut self,
        t: Self::Instant,
        edge: bool,
    ) -> Result<Self::Output, DecodingError> {
        self.event_instant(t, edge)
    }
}
