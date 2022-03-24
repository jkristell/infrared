use embedded_hal::digital::v2::InputPin;
use infrared::{
    protocol::{Rc5},
    receiver::{DefaultInput, Receiver},
    remotecontrol::{rc5::CdPlayer, Button},
};

#[test]
fn const_embedded_hal_receiver() {

    /* 
    let pin = DummyEmbeddedHalPin;
    let mut recv: Receiver<Rc5, Poll, _> = Receiver::builder()
        .rc5()
        .polled()
        .resolution(20_000)
        .pin(pin)
        .build();

    let _ = recv.poll();
    */

    let pin = DummyEmbeddedHalPin;
    let mut recv = Receiver::builder()
        .rc5()
        .remotecontrol(CdPlayer)
        .resolution(20_000)
        .pin(pin)
        .build();

    let _ = recv.event(204);
}


#[test]
fn receiver_generic() {
    let mut recv: Receiver<Rc5> = Receiver::builder()
        .rc5()
        .resolution(20_000)
        .build();

    recv.event(20, true).unwrap();

    /*
    let mut receiver: Receiver<Rc5, PeriodicPolled, _> = Builder::<Rc5>::new()
        .periodic_polled()
        .timer_resolution(20_000)
        .build();
    receiver.poll();
     */
}

#[test]
fn receiver_remote() {
    use infrared::remotecontrol::rc5;

    let mut r: Receiver<Rc5, DefaultInput, u32, Button<CdPlayer>> = Receiver::builder()
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

    let _r: Receiver<Rc5, DefaultInput, u32, Button<CdPlayer>> = Receiver::new(20_000);

    //let _r: Receiver<Rc5, DefaultInput, u32, Button<CdPlayer>> = Receiver::new(20_000);

    /*
    let _r: Receiver<Rc5, Poll, DefaultInput, u32, Button<CdPlayer>> = Receiver::builder()
        .rc5()
        .polled()
        .remotecontrol(CdPlayer)
        .build();
        */
}

struct DummyEmbeddedHalPin;

impl InputPin for DummyEmbeddedHalPin {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(true)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(false)
    }
}
