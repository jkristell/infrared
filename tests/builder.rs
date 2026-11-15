use std::convert::Infallible;

use embedded_hal::digital::{ErrorType, InputPin};

#[cfg(feature = "rc5")]
#[test]
fn receiver_remote() {
    use infrared::{
        Level,
        protocol::Rc5,
        receiver::Receiver,
        remotecontrol::{rc5, rc5::CdPlayer, Button},
    };

    let mut r: Receiver<Rc5, u32, Button<CdPlayer>> = infrared::receiver()
        .rc5()
        .remotecontrol(rc5::CdPlayer)
        .build();

    match r.event(40, Level::Space) {
        Ok(Some(event)) => {
            println!(
                "Action: {:?}, repeat: {}",
                event.action(),
                event.is_repeat()
            )
        }
        Ok(None) => (),
        Err(_err) => (),
    }

    let _r: Receiver<Rc5, u32, Button<CdPlayer>> = Receiver::new(20_000);
}

struct TestPin {
    low: bool,
}

impl ErrorType for TestPin {
    type Error = Infallible;
}

impl InputPin for TestPin {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(!self.low)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(self.low)
    }
}

#[cfg(feature = "rc5")]
#[test]
fn build_receiver_with_pin() {
    use infrared::{protocol::Rc5, PinReceiver, Receiver};

    // Rc5 command, address 20, command 9, captured at 40 kHz
    const DISTS: [u32; 20] = [
        0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
    ];

    let mut r: PinReceiver<Receiver<Rc5>, TestPin> = infrared::receiver()
        .rc5()
        .resolution(40_000)
        .build_with_pin(TestPin { low: false });

    let mut cmds = std::vec::Vec::new();
    for (i, dt) in DISTS.iter().enumerate() {
        // The receiver reads the edge state from the pin
        r.pin_mut().low = i % 2 == 0;
        if let Ok(Some(cmd)) = r.event(*dt) {
            cmds.push(cmd);
        }
    }

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].address, 20);
    assert_eq!(cmds[0].command, 9);
}
