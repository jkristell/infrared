# Infrared
A library for reading infrared signals from Rust.

## What works 
 - Receiving and decoding signals from a remote control that use the NEC protocol or the Samsung variant of it.

## Tested with
    - Tested with a st32f401re and bluepill boards
    - Vishay TSOP382 IR receiver
    - "Special for MP3" and Samsung remotes.


![Boards](http://jott.se/wordpress/wp-content/uploads/2019/06/boards_small.jpg)

## How to use it

The examples are the documentation for now. The stm32f401-interrupt is probably the one to start to look at.
As I add more features there will be breaking changes. 

## Todo
    - Imlement support for transmitting
    - Implement support for RC protocols (RC-5 and RC-6)
    - Implement support for Timer-Capture
    - More examples and utilities
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](http://jott.se/wordpress/wp-content/uploads/2019/06/remote_small.jpg)
    

