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
//!  - [`PolledReceiver`]: Polling based receiver.
//!  - [`BufferReceiver`]: Decodes buffers of edge timings.
//!  - [`ClockedReceiver`]: [`Receiver`] or [`MultiReceiver`] with its own
//!    [`Clock`], timestamping the edges itself.
//!  - `AsyncReceiver` (`async` feature): waits for edges on an
//!    embedded-hal-async pin. `async fn receive()` returns the next command.
//!
//! ### Event / Interrupt based Receiver with embedded-hal pin
//!
//! Example:
//! ```
//! use infrared::{PinReceiver,
//!     remotecontrol::rc5::CdPlayer, Command,
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
//! use infrared::{Level, Receiver, protocol::Nec};
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
//! let dt = 0;  // Time since last pin flip
//! // Level read from the pin, here an active-low IR receiver module
//! let level = Level::from_active_low(true);
//!
//! if let Ok(Some(cmd)) = receiver.event(dt, level) {
//!     println!("{} {}", cmd.address, cmd.command);
//! }
//! ```
//!
//! ### Polled
//!
//! 1. Setup a CountDown-timer at a frequency of something like 20 kHz. How to setup the timer
//!    and enable interrupts is HAL-specific but most HALs have examples showing you how to do it.
//!
//! 2. Create a Polled `infrared::PolledReceiver` with the desired Decoder state machine.
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
//! let mut receiver = infrared::receiver()
//!     .nec()
//!     .resolution(FREQ)
//!     .build_polled_with_pin(input_pin);
//!
//! // -------------------------------------------
//! // Timer interrupt handler
//! // -------------------------------------------
//!
//! if let Ok(Some(cmd)) = receiver.poll() {
//!     println!("{} {}", cmd.address, cmd.command);
//! }
//! ```
//!
//! ### Idle timeout
//!
//! A frame that is interrupted, or noise that looks like the start of a frame,
//! leaves the decoder waiting for the rest of it. Call `timeout()` once no edge
//! has been seen for the receiver's `idle_timeout()` to discard it.
//! [`PolledReceiver`] does this by itself, and [`ClockedReceiver`] checks the
//! elapsed time itself so its `timeout()` can be called at any time.
//!
//! ### Receiver with a clock
//!
//! ```
//! use infrared::{protocol::Nec, receiver::FnClock};
//! use dummy_pin::DummyPin;
//!
//! // Any time source, here a 1 MHz counter
//! let clock: FnClock<_, u32> = FnClock::new(1_000_000, || 0);
//!
//! let mut receiver = infrared::receiver()
//!     .nec()
//!     .build_clocked_with_pin(clock, DummyPin::new_high());
//!
//! // Pin interrupt: no timing bookkeeping needed
//! if let Ok(Some(cmd)) = receiver.event() {
//!     println!("{} {}", cmd.address, cmd.command);
//! }
//!
//! // Periodic timer, e.g. every 10 ms: discards interrupted frames
//! let _ = receiver.timeout();
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
//!    let r1: PinReceiver<Receiver<Rc6>, DummyPin> = PinReceiver::new(Receiver::new(40_000), pin);
//!
//!    // Periodic polled Nec Receiver
//!    let pin = DummyPin::new_low();
//!    let r2: PinReceiver<infrared::PolledReceiver<Nec>, DummyPin> =
//!        PinReceiver::new(infrared::PolledReceiver::new(40_000), pin);
//!
//!    let r3: BufferReceiver<Rc6> = BufferReceiver::with_resolution(20_000);
//!
//!    let buf: &[u32] = &[20, 40, 20];
//!    let cmd_iter = r3.iter(buf);
//!
//! ```
use core::marker::PhantomData;

use crate::{Level, Protocol};

#[cfg(feature = "async")]
mod asynch;
mod buffer;
mod builder;
mod clock;
mod decoder;
mod error;
mod iter;
mod multi;
#[cfg(feature = "embedded-hal")]
mod pin;
mod polled;
pub mod time;

#[cfg(feature = "async")]
pub use asynch::AsyncReceiver;
#[cfg(feature = "embassy")]
pub use asynch::EmbassyClock;
pub use buffer::BufferReceiver;
pub use iter::BufferIterator;
pub use builder::Builder;
#[cfg(feature = "std")]
pub use clock::StdClock;
pub use clock::{Clock, ClockedReceiver, FnClock};
pub use decoder::{DecoderBuilder, ProtocolDecoder, DecoderState};
pub use error::{DecodingError, Error};
pub use multi::{MultiReceiver, ReceiverSet};
#[cfg(feature = "embedded-hal")]
pub use pin::PinReceiver;
pub use polled::PolledReceiver;
pub use time::TimeBase;

/// Edge driven receiver
///
/// Implemented by the pinless receiver cores ([`Receiver`] and
/// [`MultiReceiver`]) so that input wrappers such as [`PinReceiver`] can be
/// generic over them.
pub trait EdgeDriven {
    /// Duration type of the time base
    type Duration;
    /// Instant type of the time base
    type Instant;
    /// Output produced on a successful decode
    type Output;

    /// Update the receiver with an edge event
    /// * `dt`: Duration since the previous edge
    /// * `level`: The input level after the edge, see [`Level`]
    fn event_edge(&mut self, dt: Self::Duration, level: Level)
        -> Result<Self::Output, DecodingError>;

    /// Update the receiver with an edge event at instant `t`
    fn event_edge_at(
        &mut self,
        t: Self::Instant,
        level: Level,
    ) -> Result<Self::Output, DecodingError>;

    /// Time without edges after which the input is considered idle
    fn idle_timeout(&self) -> Self::Duration;
}

/// Receiver that can be told that its input has gone idle
///
/// Implemented by all receiver cores and wrappers. Calling
/// [`timeout`](Timeout::timeout) when no edge has been seen for at least the
/// receiver's idle timeout discards partially received frames, so that noise
/// or an interrupted transmission doesn't linger in the decoder until the next
/// frame arrives.
pub trait Timeout {
    /// Output produced on a successful decode
    type Output;

    /// Notify the receiver that the input has been idle
    ///
    /// Returns a command if the protocol completes its frames on silence.
    fn timeout(&mut self) -> Result<Self::Output, DecodingError>;
}

/// Poll driven receiver
///
/// Implemented by [`PolledReceiver`] so that input wrappers such as
/// [`PinReceiver`] can be generic over it.
pub trait PollDriven {
    /// Output produced on a successful decode
    type Output;

    /// Update the receiver with the current pin state
    fn poll_edge(&mut self, level: Level) -> Result<Self::Output, DecodingError>;
}

/// Event based Receiver
///
/// The core protocol decoder state machine. Hardware independent: edge events
/// are fed to it via [`event`](Receiver::event) or
/// [`event_at`](Receiver::event_at). Combine it with
/// [`PinReceiver`] to let it read the edges from an embedded-hal pin.
pub struct Receiver<
    Proto: DecoderBuilder<Time>,
    Time: TimeBase = u32,
    Cmd: From<Proto::Cmd> = <Proto as Protocol>::Cmd,
> {
    /// Decoder data
    pub(crate) decoder: Proto::Decoder,
    /// Previous instant, used by `event_at`
    prev_instant: Time::Instant,
    /// Type of the final command output
    pub(crate) cmd: PhantomData<Cmd>,
}

impl<Proto, Time, Cmd> Receiver<Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<Proto::Cmd>,
{
    /// Create a new Receiver with the given clock resolution (in Hz)
    pub fn new(freq: u32) -> Self {
        Receiver {
            decoder: Proto::build(freq),
            prev_instant: Time::ZERO_INSTANT,
            cmd: PhantomData,
        }
    }

    /// Update the receiver
    /// * `dt`: Duration since the previous edge
    /// * `level`: The input level after the edge, see [`Level`]
    pub fn event(&mut self, dt: Time::Duration, level: Level) -> Result<Option<Cmd>, DecodingError> {
        self.decoder
            .event_total(dt, level)
            .map(|cmd| cmd.map(Into::into))
    }

    /// Notify the receiver that the input has been idle for at least
    /// [`idle_timeout`](Receiver::idle_timeout)
    ///
    /// Discards any partially received frame. Calling it while no frame is
    /// in progress is harmless.
    pub fn timeout(&mut self) -> Result<Option<Cmd>, DecodingError> {
        self.decoder
            .timeout_total()
            .map(|cmd| cmd.map(Into::into))
    }

    /// Time without edges after which the input is considered idle
    ///
    /// Derived from the protocol timing: twice the longest valid symbol.
    pub fn idle_timeout(&self) -> Time::Duration {
        self.decoder.spans().idle_timeout()
    }

    /// Update the receiver with an edge event at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_at(
        &mut self,
        t: Time::Instant,
        level: Level,
    ) -> Result<Option<Cmd>, DecodingError> {
        let dt = Time::checked_sub(t, self.prev_instant).unwrap_or(Time::ZERO_DURATION);
        self.prev_instant = t;

        self.event(dt, level)
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

impl<Proto, Time, Cmd> EdgeDriven for Receiver<Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<Proto::Cmd>,
{
    type Duration = Time::Duration;
    type Instant = Time::Instant;
    type Output = Option<Cmd>;

    fn event_edge(
        &mut self,
        dt: Self::Duration,
        level: Level,
    ) -> Result<Self::Output, DecodingError> {
        self.event(dt, level)
    }

    fn event_edge_at(
        &mut self,
        t: Self::Instant,
        level: Level,
    ) -> Result<Self::Output, DecodingError> {
        self.event_at(t, level)
    }

    fn idle_timeout(&self) -> Self::Duration {
        Receiver::idle_timeout(self)
    }
}

impl<Proto, Time, Cmd> Timeout for Receiver<Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<Proto::Cmd>,
{
    type Output = Option<Cmd>;

    fn timeout(&mut self) -> Result<Self::Output, DecodingError> {
        Receiver::timeout(self)
    }
}
