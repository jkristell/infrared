use embedded_hal::digital::v2::InputPin;
use infrared::{
    protocol::{NecSamsung, Rc5},
    receiver::{ConstReceiver, DefaultInput, Event, Poll, Receiver},
    remotecontrol::{rc5::CdPlayer, Button},
};

#[test]
fn const_embedded_hal_constreceiver() {
    let pin = DummyEmbeddedHalPin;

    let mut recv = Receiver::builder()
        .rc5()
        .polled()
        .pin(pin)
        .build_const::<1_000_000>();

    recv.poll().unwrap();

    let pin = DummyEmbeddedHalPin;
    let mut recv: ConstReceiver<Rc5, Event, _, 1_000_000> = Receiver::builder()
        .rc5()
        .event_driven()
        .pin(pin)
        .build_const();

    let _ = recv.event(100);
}

#[test]
fn const_embedded_hal_receiver() {
    let pin = DummyEmbeddedHalPin;

    let mut recv: Receiver<Rc5, Poll, _> = Receiver::builder()
        .rc5()
        .polled()
        .resolution(20_000)
        .pin(pin)
        .build();

    let _ = recv.poll();

    let pin = DummyEmbeddedHalPin;
    let mut recv = Receiver::builder()
        .rc5()
        .remotecontrol(CdPlayer)
        .event_driven()
        .resolution(20_000)
        .pin(pin)
        .build();

    let _ = recv.event(204);

    let _recv = Receiver::builder()
        .protocol::<NecSamsung>()
        .event_driven()
        .build_const::<400_000>();
}

#[test]
fn receiver_iterator() {
    let mut recv: Receiver<Rc5, Event, _> = Receiver::builder()
        .rc5()
        .event_driven()
        .resolution(20_000)
        .buffer(&[])
        .build();

    recv.set_buffer(&[20, 40, 20, 40]);

    // Iterate through the commands
    let _i = recv.iter();
}

#[test]
fn receiver_generic() {
    let mut recv: Receiver<Rc5, Event, _> = Receiver::builder()
        .rc5()
        .event_driven()
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

    let mut r: Receiver<Rc5, Event, DefaultInput, Button<CdPlayer>> = Receiver::builder()
        .event_driven()
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

    let _r: Receiver<Rc5, Event, DefaultInput, Button<CdPlayer>> = Receiver::new(20_000);

    let _r: Receiver<Rc5, Poll, DefaultInput, Button<CdPlayer>> = Receiver::new(20_000);

    let _r: Receiver<Rc5, Poll, DefaultInput, Button<CdPlayer>> = Receiver::builder()
        .rc5()
        .polled()
        .remotecontrol(CdPlayer)
        .build();
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
