[![crates.io version](https://meritbadge.herokuapp.com/infrared)](https://crates.io/crates/infrared)
[![docs.rs](https://docs.rs/infrared/badge.svg)](https://docs.rs/infrared)

# Infrared

Rust library for using Infrared hardware decoders (For example a Vishay TSOP* decoder),
enabling remote control support for embedded project.

This library aims for to be useful with the any MCU hal that implements the embedded-hal traits,
and at the same time provide functionality for using it with more efficient implementation
such as input capture, and be useful in host applications (such as Blipper).


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


![Boards](https://jott.se/txrx_setup.jpg)

## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](https://jott.se/remote_small.jpg)
    
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
