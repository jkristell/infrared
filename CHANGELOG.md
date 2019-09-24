# Changelog

## TODO
 - Add tests for all type of receivers
 - Ensure that examples work
 - Test transmitter
 - Maybe remove Transmitter altogether to be able to 
   add it back in point release?


## [0.4.0] - 2019-09-XX
 - Rewrote the NEC receiver
 - Added protocol-dev feature for easier protocol debugging
 - Lots of API breakage :)
   * Receiver::event() is now split in to ::sample() and ::sample_edge()
 

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