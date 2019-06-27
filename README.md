# Infrared
A library for interacting with IR-based devices for embedded Rust.

## Status

### The good 
 - Receiving and decoding signals Remote controls that use the NEC protocol,
   or the Samsung variant of it should work.
 - Adding a mapping for a new Remote is doable
 
### The bad
  - The API is to be considered WIP and will evolve as find add new
  features and correct all mistakes done by me previously :).
  - The tools for capturing remotes is not there yet
  - Only supports NEC
  
## Tested with
    - Tested with a st32f401re and bluepill boards
    - Vishay TSOP382 IR receiver
    - "Special for MP3" and Samsung remotes.


![Boards](http://jott.se/wordpress/wp-content/uploads/2019/06/boards_small.jpg)

## How to use it

The examples are the documentation for now. The stm32f401-interrupt is probably the one to start to look at.
As I add more features there will be breaking changes. 

## Example

```Rust

let nec: NecReceiver<SpecialForMp3> = NecReceiver::new(20_000);

```

## Todo
    - Imlement support for transmitting
    - Implement support for RC protocols (RC-5 and RC-6)
    - Implement support for Timer-Capture
    - More examples and utilities
    
## References

 * https://www.sbprojects.net/knowledge/ir/nec.php
 * https://www.vishay.com/docs/82491/tsop382.pdf

![Remote](http://jott.se/wordpress/wp-content/uploads/2019/06/remote_small.jpg)
    

