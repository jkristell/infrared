# Infrared
A library for using infrared remote controls with Rust.

## Status

### Supported protocols
 - The NEC Protocol and the Samsung variant of it
 - Philips Rc5
 - Philips Rc6

### Examples
 - Receiving, NEC, Rc5 and Rc6 ``examples/bluepill-receiver``
 - Transmitting NEC ``examples/bluepill-tx``
  
## Tested with
  - Tested with bluepill board
  - Vishay TSOP382 IR receiver
  - Various ir leds
  - NEC Generic "Special for MP3" and Samsung remotes
  - Rc6 tested with a Philips Bluray player remote


![Boards](http://jott.se/wp-content/uploads/2019/09/txrx_setup.jpg)


## How to use it
The examples are the documentation right now.

## Near time goals
    - Implement support for more remotes of different kinds
    - Investigate if the pwm traits from Embedded-hal can be used
    - Hw Timer-Capture based example
    - Better tracer/capture application so that remotes can be
     cloned easily
    - More utilities
    
## Long time goals
USB/Network support to be able to create "universal remote control" types of applications.
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](http://jott.se/wp-content/uploads/2019/09/remote_small.jpg)
    
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

