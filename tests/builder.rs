use std::convert::Infallible;
use embedded_hal::digital::{ErrorType, InputPin};

#[cfg(feature = "rc5")]
#[test]
fn receiver_remote() {
    use infrared::{
        protocol::Rc5,
        receiver::{NoPin, Receiver},
        remotecontrol::{rc5, rc5::CdPlayer, Button},
    };

    let mut r: Receiver<Rc5, NoPin, u32, Button<CdPlayer>> = infrared::receiver()
        .rc5()
        .remotecontrol(rc5::CdPlayer)
        .build();

    match r.event(40, false) {
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

    let _r: Receiver<Rc5, NoPin, u32, Button<CdPlayer>> = Receiver::new(20_000);
}

struct DummyEmbeddedHalPin;

impl ErrorType for DummyEmbeddedHalPin { type Error = Infallible; }

impl InputPin for DummyEmbeddedHalPin {

    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
