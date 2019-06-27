# Infrared
A library for using remote controls with embedded Rust!

## Status

### The good 
 - Receiving and decoding signals Remote controls that use the NEC protocol,
   or the Samsung variant of it should work.
 - Adding a mapping for a new Remote is doable
 
### The bad
  - The API is to be considered WIP and will evolve as find add new
  features and correct all mistakes done by me previously :).
  - The tools for capturing remotes is not great yet
  - Only supports the NEC protocol
  
## Tested with
    - Tested with a st32f401re and bluepill boards
    - Vishay TSOP382 IR receiver
    - "Special for MP3" and Samsung remotes.


![Boards](http://jott.se/wordpress/wp-content/uploads/2019/06/boards_small.jpg)

## How to use it

The examples are the documentation for now. The stm32f401-interrupt is probably the one to start to look at.
Beware that as I add more features there will be breaking changes.
But the examples will be kept working. 

## Near time goals
    - Imlement support for transmitting
    - Implement support for RC protocols (RC-5 and RC-6)
    - Hw Timer-Capture based example
    - Better tracer/capture application so that remotes can be
     cloned easily
    - More utilities
    
## Long time goals
 USB/Network support to be able to create universal remote control type of applications.
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](http://jott.se/wordpress/wp-content/uploads/2019/06/remote_small.jpg)
    

