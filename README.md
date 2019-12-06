# Infrared
A library for using infrared remote controls with Rust.

## Status

### Supported protocols
 - The NEC Protocol and the Samsung variant of it
 - Philips Rc5
 - Philips Rc6
 - "Samsung BluRay Player protocol". Please let know if you know what it really is called :)

### Examples
 - The ``examples/stm32f103-examples`` contains various examples for receiving and transmitting infrared with the
  blue pill board
 - [Blipper](https://github.com/jkristell/blipper) - An application for working
 with transmitters and receivers from a host computer
 - [Tutorial 1](https://jott.se/blog/infrared)
  
## Tested with
 - Tested with bluepill board
 - Vishay TSOP382 IR receiver
 - Various ir leds
 - NEC Generic "Special for MP3" and Samsung remotes
 - Rc6 tested with a Philips Bluray player remote
 - Rc5 tested with a Marantz CD player remote


![Boards](https://jott.se/wp-content/uploads/2019/09/txrx_setup.jpg)

## How to use it
See the examples.

## Near time goals
 - Implement transmit for Rc6 and Samsung BluRay Player protocol
 - Implement support for more remotes of different kinds
 - Hw Timer-Capture based example

## Long time goals
USB/Network support to be able to create "universal remote control" types of applications.
Work started in [Blipper](https://github.com/jkristell/blipper) repository.
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](https://jott.se/wp-content/uploads/2019/09/remote_small.jpg)
    
## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.

