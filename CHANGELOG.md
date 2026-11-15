# Changelog

## [0.15.0] - Unreleased
 - Command model, aligned with the terminology of other IR libraries (IRremote, Flipper, IRP):
   - All commands use `address`/`command`/`repeat`/`toggle` fields (was `addr`/`cmd`), and derive `Copy`, `Clone`, `PartialEq`, `Eq` and `Hash`
   - `Command` and `AddressedCommand` merged into `protocol::Command` with `PROTOCOL`, `address()`, `command()`, `is_repeat()`, `raw()` and `from_parts()` (was `create()`). The `cmd` module is removed; `Command` and `Message` are re-exported from the crate root
   - `AnyCommand` is now `Message`
   - `ProtocolId`: `From<u8>` and the `Unknown` variant replaced by `TryFrom<u8>` (`UnknownProtocol` error); added `as_str()` and `Display`
 - Protocol renames:
   - `Sbp` is now `Samsung36` (its name in IRremote and the IRP database), with the `samsung36` feature, `protocol::samsung36` module and `Builder::samsung36()`
   - `SamsungNec` is now `Samsung32`, `Builder::nec_samsung()` is `samsung32()`
   - `Nec16` is now `NecExtended`, `Builder::nec16()` is `nec_extended()`
   - `NecDebug`/`NecDebugCmd` are now `NecRaw`/`NecRawCommand`
   - `Builder::nec_apple()` is now `apple_nec()`; `ProtocolId` variants renamed to match the types
 - New `Level` enum (`Mark`/`Space`) replaces the `edge: bool` and `level: bool` arguments, with `Level::from_active_low(pin.is_low())` and `from_active_high()` helpers. The duration now always comes first: `ProtocolDecoder::event(dt, level)` (was `event(edge, dt)`)
 - Errors: `Error<PinErr>` is now `Error::Decode(DecodingError)` or `Error::Pin(PinErr)` (was a copy of the `DecodingError` variants plus `Hal`). Removed the unused `DecodingError::RemoteControlError` and made `DecodingError` `#[non_exhaustive]`. Both implement `Display` and `core::error::Error`
 - `Samsung36Command` no longer has a `valid` field: frames with a bad checksum are a `DecodingError::Validation`
 - `DenonCommand` is decoded as Denon-K (Kaseikyo): 12 bit `address` and 8 bit `command` instead of the raw `bits`. The vendor id and parity are validated
 - Apple NEC: commands can now be encoded (`pack()` panicked), fixed an overflow in `command()`, and the device id is now the address
 - `BufferIterator` skips frames that fail to decode instead of ending the iteration
 - Idle timeout handling: `timeout()` on `Receiver`, `MultiReceiver`, `PolledReceiver` and `PinReceiver` (new `Timeout` trait) discards partially received frames once the input has been idle for `idle_timeout()`, which is derived from the protocol timing. `PolledReceiver` does this automatically. `ProtocolDecoder::timeout` lets protocols whose frames end on silence finish the frame. `EdgeDriven` gained `idle_timeout()`
 - Clock support: new `Clock` trait with `FnClock` (adapts any time source, e.g. a hardware timer, RTIC monotonic or fugit instant) and `StdClock` (new `std` feature). `ClockedReceiver` timestamps edges itself: `event(level)`, a self-checking `timeout()`, and `poll(level)` that detects edges and idle input from the level alone. With a pin, `PinReceiver::event()` takes no arguments. Builder: `build_clocked(clock)` and `build_clocked_with_pin(clock, pin)`
 - Async receiver: new `async` feature with `AsyncReceiver`, which waits on an embedded-hal-async `Wait` pin and uses a `DelayNs` for the idle timeout. `receive().await` returns the next command and skips decode errors; `next().await` returns every decoder result. The `embassy` feature adds `EmbassyClock` and `AsyncReceiver::embassy(pin)`. Builder: `build_async(clock, pin, delay)` and `build_embassy(pin)`
 - `InfraMonotonic` has a new required method `to_micros()`
 - Sender rework:
   - `ProtocolEncoder` no longer has a `FREQ` const parameter. Encoders produce mark/space durations in µs (with `MAX_PULSES`), and the senders convert them at their runtime resolution: `TickSender::new(resolution)`, `PwmSender::new(pwm, resolution)`, `PwmSender::with_duty_fraction(pwm, resolution, num, denom)`
   - The buffer size parameter defaults to 128 (`DEFAULT_BUFFER_SIZE`) and is checked against the protocol at compile time. `PwmSender<Pwm, FREQ, N>` is now `PwmSender<Pwm, N = 128>`
   - `PwmSender::load` returns `Err(Busy)` instead of silently ignoring a command during a transmission. `TickSender::load_command::<P, F>` is now `TickSender::load::<P>`
   - New `AsyncSender` (`async` feature): `send::<P>(&cmd).await` with a `SetDutyCycle` pwm and an embedded-hal-async delay; `AsyncSender::embassy(pwm)` with the `embassy` feature. The carrier is turned off if a send is cancelled. The `async` feature now enables `embedded-hal`
   - New Denon and Samsung36 encoders
   - Fixed the final mark of NEC and RC5 frames being cut to a single tick, which IR receiver modules don't detect
   - NEC repeat commands are now encoded as short repeat frames (Samsung NEC resends the full frame, see `NecCommandVariant::SHORT_REPEAT`). `NecCommandVariant` has a new required `repeat()` method
   - Fixed overflow and precision loss when converting pulse lengths to ticks (e.g. RC6 above ~9.6 MHz)
 - Fixed the Denon decoder dropping bit 47
 - Examples: new async `embassy-rp-pico` example with a receiver and a sender
 - Plain `u32`/`u64` time counters may now wrap around: elapsed time is computed with wrapping arithmetic
 - Rewrote the embedded-hal sender for embedded-hal 1.0 (`SetDutyCycle`), with configurable carrier duty cycle
 - Type renames for a cleaner API: `PeriodicPoll` → `PolledReceiver`, the `EdgeReceiver`/`PollReceiver` traits → `EdgeDriven`/`PollDriven`, `ReceiverWrapper` → `ReceiverSet`, `receiver::State` → `DecoderState`, the embedded-hal sender → `PwmSender`, `RemoteControlModel` → `RemoteControl`, `AddressCommand` → `AddressedCommand`
 - Iterator cleanup: `BufferReceiver::iter` takes `&self`; `BufferIterator` is exported and implements `FusedIterator`; removed the unused `iter_with` and the hand-rolled `PulseIterator` (slice iterator used instead)
 - Fixed `PulseBuffer::get` reading stale data past the loaded pulse train, which made `TickSender` toggle rapidly after each transmission
 - Terminology cleanup:
   - `BufferInputReceiver` is now `BufferReceiver`, and `with_frequenzy` is `with_resolution`
   - `PulsedataBuffer`/`PulsedataSender` are now `PulseBuffer`/`TickSender`
   - `Builder::frequency()` is now `Builder::resolution()`
   - The pulse period enums are now named symbols: `NecSymbol`, `DenonSymbol`, `SbpSymbol` (they classify mark+space periods, not pulse widths)
   - Documented the edge semantics: `edge == true` means start of an IR mark (low pin on an active-low receiver module)
 - Updated to edition 2024 and declared `rust-version = "1.85"` (MSRV), checked in CI
 - New tests: SBP decoding (previously untested), Denon regression values, fugit u64 receiver, `PinReceiver` polling, and remote control button mapping
 - `PeriodicPoll` is now a pinless core like `Receiver`: `poll(edge)` takes the sampled pin state, and pin based polling goes through `PinReceiver` (`PeriodicPoll::with_pin` returns one). New `PollReceiver` trait
 - Removed `NoPin`
 - CI: added a clippy check
 - docs.rs now builds with all features enabled
 - New tests: `PinReceiver` via the builder, and `MultiReceiver` (duration and instant based)
 - Receiver API rework: `Receiver` is now a hardware independent core (no pin, 3 generic parameters). The new `PinReceiver` wraps any edge-driven receiver with an embedded-hal `InputPin`
 - `MultiReceiver` is pinless too, gained instant based eventing, and shares the pin handling with `Receiver` through `PinReceiver`
 - New `EdgeReceiver` trait implemented by `Receiver` and `MultiReceiver`, intended as the extension point for future input drivers (e.g. async)
 - Builder: `.pin(pin).build()` is now `.build_with_pin(pin)`; added `.build_polled_with_pin(pin)`
 - MultiReceiver: `pin()` now returns a shared reference; use `pin_mut()` for a mutable one (matching `Receiver`); added `release()`
 - Fixed `Span::overlaps` missing the fully-contained case
 - Ported to embedded-hal 1.0 (#106)
 - Updated defmt to 1.0 (#115)
 - Removed the `embedded` feature. Use `embedded-hal` and `fugit`, both enabled by default (#117)
 - Fixed compile errors in several non-default feature combinations (#117)
 - Enabling both `defmt` and `log` is now allowed; defmt takes precedence (#117)
 - Nec: added repeat state to NecDebugCmd (#105)
 - Nec: increased tolerances for bit pulses (#107)
 - Added AnyCommand support for AddressCommand (#93, #96)
 - Removed the Capture protocol (#92)
 - Examples: updated bluepill and rp-pico examples for embedded-hal 1.0; removed the arduino and sender examples (#114)

## [0.14.2] - 2023-01-22
 - add a to_str() method on Action

## [0.14.0] - 2022-05-05
 - Added support fugit and made reworked the time handling
 - Lots of internal updates
 - Removed the ConstReceiver type. Use a Receiver with Fugit instead.
 - Renamed NecApple to AppleNec and NecSamsung to SamsungNec
 - Added rp-pico example.

## [0.13.0] - 2021-09-28
 - Added the ability to specify a remote control with a Receiver
 - Button renamed to Action
 - New Button type is a wrapper around a Command and an Action
 - Switched from usize to u32 as the integer type in the APIs
 - Rewrote MultiReceiver

## [0.12.0] - 2021-07-05
 - Merged all the Receiver types into one.
 - Lots of API changes
 - Lots of refactoring

## [0.11.0] - 2021-03-13
 - The pulse lengths, for both receiver and sender are now calculated when the state machines are created.
 - Internal refactoring of both sender and receiver
 - AVR: Rc5 and Rc6 Receiver verified to work
 - AVR: There is a problem with AVR and Nec decoding. The issue is in the LLVM code generation. please use the 'avr' branch until that's resolved.

## [0.10.0] - 2021-02-02
 - Added some support for Denon 48 bit protocol
 - Reworked the BufferReceiver API

## [0.9.0] - 2021-01-28
 - Added support the Apple variant of the Nec Protocol and added a basic AppleRemote (Thanks @jhillyerd)
 - Fixed the repeat detection on the NEC receiver. The NEC commands now have a boolean repeat flag to let the
   user detect repeats.
 - Renamed HalSender to Sender and move hal::{Sender, Receiver} to the root of the crate
 - Lots of various internal refactoring, mostly on the NEC protocol

## [0.8.0] - 2020-12-29
 - Added HalSender and removed the protocol specific senders
 - Added Rc6 transmit support
 - Updated Nec and Rc5 transmit support
 - moved examples to the infrared-examples repo
 - Added more tests

## [0.7.0] - 2020-09-12
 - Lots of breaking API changes and internal cleanups
 - New Receiver types
 - Bugfixes for Rc5 and Rc6 receivers. Should work much better now.
 - Added Usb media keyboard example.

## [0.6.0] - 2019-12-06
 - Split decode state machine and hal
 - New InfaredReceiver type
 - Lots of API breakage

## [0.5.0] - 2019-11-07
 - Added (optional) embedded-hal support
 - Added support for NEC commands with 16 bit addresses
 - Added support for Samsung BluRay Player protocol
 - Refactored remotes
 - Cleaned up examples

## [0.4.1] - 2019-09-29
 - Added support for Rc5 transmit

## [0.4.0] - 2019-09-26
 - Rewrote the receivers
 - Examples updated
 - Added protocol-dev feature for easier protocol debugging
 - Added some basic testing
 - Lots of API breakage
 - Mentioned Blipper

## [0.3.2] - 2019-09-21
 - Added support for Philips Rc5 Protocol

## [0.3.1] - 2019-09-16
 - Added support for Philips Rc6-0 Protocol
 - Added trace support

## [0.3.0] - 2019-08-04
 - Added transmit support
 - Added bluepill transmit example
 - Lots of cleanups
 - Removed stm32f401 examples

## [0.2.0] - 2019-06-26

 - Support for Samsung version of the NEC protocol
 - Started work on supporting mappings for remotes
 - Bluepill board example
 - Added an example on how to use external interrupt

## [0.1.0] 

Initial release 