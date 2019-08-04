# Infrared
A library for using infrared remote controls with Rust.

## Status

### The good 
 - Receiving and decoding signals from remote controls that use the NEC protocol,
   or the Samsung variant of it, should work
 - Transmitting works! see ``examples/bluepill-tx``
 - Adding support for more NEC remotes is doable
 
### The bad
  - The API is to be considered WIP and will evolve as I add new
  features and correct all mistakes done by me previously :).
  - No tool for capturing remotes yet
  - Only supports the NEC protocol
  
## Tested with
  - Tested with bluepill board
  - Vishay TSOP382 IR receiver
  - Various ir leds
  - "Special for MP3" and Samsung remotes


![Boards](https://jott.se/wordpress/wp-content/uploads/2019/08/txrx_setup.jpg)

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

![Remote](https://jott.se/wordpress/wp-content/uploads/2019/06/remote_small.jpg)
    

