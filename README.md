# Infrared
A library for using infrared remote controls with Rust.

## Status

### Supported protocols
 - The NEC Protocol and the Samsung variant of it
 - Philips Rc6

### Examples
 - Transmitting works! see ``examples/bluepill-tx``
  
## Tested with
  - Tested with bluepill board
  - Vishay TSOP382 IR receiver
  - Various ir leds
  - NEC "Special for MP3" and Samsung remotes
  - Rc6 Tested with a Philips Bluray player remote


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
    

