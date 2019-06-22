# Infrared
A library for decoding infrared signals from receivers for Embedded Rust.

## What works 
 - Receiving and decoding signals from a remote control that use the NEC protocol

## Tested with

Tested with a st32f401re board, a Vishay TSOP382 and "Special for MP3" remote. The example code is available  [Here](examples/polling)


![Boards](http://jott.se/wordpress/wp-content/uploads/2019/06/boards_small.jpg)


## Todo
    - Test more NEC remotes
    - Implement support for extended NEC
    - Imlement support for transmitting
    - Implement support for RC protocols (RC-5 and RC-6)
    - Implement support for Timer-Capture
    - More examples and utilities
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
    
![Remote](http://jott.se/wordpress/wp-content/uploads/2019/06/remote_small.jpg)
    

