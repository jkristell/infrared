use embedded_hal::digital::v2::InputPin;
use infrared::protocol::NecSamsung;
use infrared::protocol::Rc5;
use infrared::receiver::{Builder, ConstReceiver, Event, Poll, Receiver};

#[test]
fn const_embedded_hal_constreceiver() {
    let pin = DummyEmbeddedHalPin;

    let mut recv = Builder::<Rc5>::new()
        .polled()
        .pin(pin)
        .build_const::<1_000_000>();

    recv.poll().unwrap();

    let pin = DummyEmbeddedHalPin;
    let mut recv: ConstReceiver<Rc5, Event, _, 1_000_000> =
        Builder::<Rc5>::new().event_driven().pin(pin).build_const();

    let _ = recv.event(100);
}

#[test]
fn const_embedded_hal_receiver() {
    let pin = DummyEmbeddedHalPin;

    let mut recv: Receiver<Rc5, Poll, _> = Builder::<Rc5>::new()
        .polled()
        .resolution(20_000)
        .pin(pin)
        .build();

    let _ = recv.poll();

    let pin = DummyEmbeddedHalPin;
    let mut recv: Receiver<Rc5, Event, _> = Builder::<Rc5>::new()
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

    //let mut r: BufferReceiver<Rc5> = BufferReceiver::with_buf(20_000, &[]);
}

#[test]
fn receiver_generic() {
    let mut recv: Receiver<Rc5, Event, _> = Builder::<Rc5>::new()
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
