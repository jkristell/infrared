//! # Infrared
//!
//! Rust library for using Infrared hardware decoders (For example a Vishay TSOP* decoder),
//! enabling remote control support for embedded project.
//!
//! This library aims for to be useful with the any MCU hal that implements the embedded-hal traits,
//! and at the same time provide functionality for using it with more efficient implementation
//! such as input capture, and be useful in host applications (such as Blipper).
//!
//! ## Design
//!
//! ### Receivers
//!
//! ```text
//! +------------------------+---------------------+
//! | hal::PeriodicReceiver  | hal::EventReceiver  |  Embedded hal based receivers
//! |------------------------+---------------------|
//! |    PeriodicReceiver    |    EventReceiver    |  Generic Receivers
//! |------+--------+--------+---+-----+-----+-----|
//! | Nec  | Nec16  | NecSamsung | Rc5 | Rc6 | Sbp |  Protocol state machines
//! +------+--------+------------+-----+-----+-----+
//! ```
//!
//! ## Using Infrared with embedded-hal
//!
//! ### Polled
//!
//! Right now, this is the easiest way to setup Infrared to work with any embedded-hal based board.
//! 1. Setup a CountDown-timer at a frequency of something like 20 kHz. How to setup the timer
//! and enable interrupts is HAL-specific but most HALs have examples showing you how to do it.
//!
//! 2. Create a `hal::PeriodicReceiver` with the desired Decoder state machine.
//!
//! Example:
//! ```ignore
//! use infrared::{hal::PeriodicReceiver, protocols::Rc5}
//! use embedded_hal::digital::v2::InputPin;
//!
//! const SAMPLERATE: u32 = 20_000;
//! let pin = ... // Setup the input pin connected to the infrared receiver
//! let mut recv: PeriodicReceiver<Rc5, PINTYPE> = PeriodicReceiver::new(pin, SAMPLERATE);
//! ```
//!
//! 3. In the timer interrupt handler for the timer `poll` the receiver and wait for it to
//! successfully detect a command
//!
//! ```ignore
//! if let Ok(Some(cmd)) = recv.poll() {
//!     rprintln!("{} {}", cmd.address(), cmd.data());
//! }
//! ```
//!
//! There is also support for receiving and decoding the pulse train to a known remote control
//!
//! ```ignore
//! use infrared::{remotecontrol::Button, remotes::rc5::Rc5CdPlayer};
//!
//! if let Ok(Some(button)) = recv.poll_button::<Rc5CdPlayer>() {
//!     match button {
//!         Button::Play => ... // Handle play,
//!         Button::Stop => ... // Handle stop
//!         ...
//!     }
//! }
//! ```
//!
//! #### Evented
//!
//! The library could also be used with external interrupt if you have a way of keeping track
//! time between the interrupts.
//!
//! The `receiver_exti` example, shows a way of doing it using a timer method found in
//! the stm32f1xx-hal (non embedded-hal). Another way would be to have a
//! monotonic timer running and using that for keeping track of the time between the edges.
//!
//! ## Examples
//!
//! In the examples directory in the github repo, there are fully working examples
//! of different kind of receivers
//!
//! * `receiver`: HAL Periodic receiver example
//! * `receiver_exti`: HAL EventReceiver using external interrupts
//! * `multireceiver`: receiver for multiple protocols
//! * `mediakeyboard`: USB hid media keyboard, RTIC based Media keyboard controlled by remote control.
//! * `sender`: Send example
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
#[macro_use]
extern crate std;

pub mod protocols;

mod recv;
pub use recv::{EventReceiver, PeriodicReceiver, ReceiverSM};

pub mod bufrecv;

pub mod sender;

mod remotecontrol;
pub use remotecontrol::{Button, DeviceType, RemoteControl};

mod cmd;
#[doc(inline)]
pub use cmd::{Protocol};

#[cfg(feature = "remotes")]
pub mod remotes;

#[cfg(feature = "embedded-hal")]
pub mod hal;

/// Time base
pub const TIMEBASE: u32 = 1_000_000;

pub trait PulseLengths {
    fn encode(&self, buf: &mut [u16]) -> usize;
}