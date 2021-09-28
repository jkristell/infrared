# Changelog

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