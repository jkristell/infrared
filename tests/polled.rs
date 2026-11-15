#[cfg(feature = "rc5")]
#[test]
fn polled_rc5() {
    use infrared::{
        Level,
        protocol::{rc5::Rc5Command, Rc5},
        PolledReceiver,
    };

    // Rc5 cmd data sampled at 40 kHz
    let data = [
        57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36,
        71, 36, 26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36,
        71, 36,
    ];

    let mut recv: PolledReceiver<Rc5> = infrared::receiver().rc5().resolution(40_000).build_polled();

    let mut pinstate = Level::Space;

    let mut res = None;

    for n in data {
        for _p in 0..n {
            if let Ok(Some(cmd)) = recv.poll(pinstate) {
                res = Some(cmd);
            }
        }
        pinstate = pinstate.toggle();
    }

    assert_eq!(
        res,
        Some(Rc5Command {
            address: 20,
            command: 1,
            start: 3,
            toggle: false
        })
    );
}

#[cfg(feature = "rc5")]
#[test]
fn polled_rc5_with_pin() {
    use core::convert::Infallible;

    use embedded_hal::digital::{ErrorType, InputPin};
    use infrared::{
        protocol::{rc5::Rc5Command, Rc5},
        PolledReceiver,
    };

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

    // Rc5 cmd data sampled at 40 kHz
    let data = [
        57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36,
        71, 36, 26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36,
        71, 36,
    ];

    let mut recv = PolledReceiver::<Rc5>::with_pin(40_000, TestPin { low: false });

    let mut res = None;
    for n in data {
        for _ in 0..n {
            if let Ok(Some(cmd)) = recv.poll() {
                res = Some(cmd);
            }
        }
        let state = recv.pin().low;
        recv.pin_mut().low = !state;
    }

    assert_eq!(
        res,
        Some(Rc5Command {
            address: 20,
            command: 1,
            start: 3,
            toggle: false
        })
    );
}
