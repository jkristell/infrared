#![cfg(all(feature = "rc5", feature = "embedded-hal"))]

use core::convert::Infallible;

use embedded_hal::pwm::{ErrorType, SetDutyCycle};
use infrared::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::BufferReceiver,
    sender::{PwmSender, Status},
};

const FREQ: u32 = 40_000;

#[derive(Default)]
struct MockPwm {
    duty: u16,
    writes: usize,
}

impl ErrorType for MockPwm {
    type Error = Infallible;
}

impl SetDutyCycle for MockPwm {
    fn max_duty_cycle(&self) -> u16 {
        1000
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error> {
        self.duty = duty;
        self.writes += 1;
        Ok(())
    }
}

#[test]
fn send_receive_roundtrip() {
    let mut sender: PwmSender<MockPwm, FREQ, 96> = PwmSender::new(MockPwm::default());
    assert!(sender.is_idle());

    let cmd = Rc5Command::new(20, 9, false);
    sender.load::<Rc5>(&cmd);
    assert!(!sender.is_idle());

    // A second load while busy must not replace the pending command
    sender.load::<Rc5>(&Rc5Command::new(1, 1, false));

    // Drive the sender and record the tick index of every carrier on/off edge
    let mut edges = vec![];
    let mut carrier_on = false;
    for tick in 0..2000u32 {
        let status = sender.tick().unwrap();
        let on = status == Status::Transmit(true);
        if on != carrier_on {
            edges.push(tick);
            carrier_on = on;
        }
    }
    assert!(sender.is_idle());
    assert!(!carrier_on);

    // Convert edge timestamps to edge distances
    let mut dists = vec![edges[0]];
    dists.extend(edges.windows(2).map(|w| w[1] - w[0]));

    // Feed the pulse train back into a receiver
    let receiver = BufferReceiver::<Rc5>::with_resolution(FREQ);
    let cmds: Vec<_> = receiver.iter(&dists).collect();

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].addr, 20);
    assert_eq!(cmds[0].cmd, 9);
}

#[test]
fn duty_cycle_and_write_behavior() {
    let mut sender: PwmSender<MockPwm, FREQ, 96> =
        PwmSender::with_duty_fraction(MockPwm::default(), 1, 3);

    let cmd = Rc5Command::new(1, 2, false);
    sender.load::<Rc5>(&cmd);

    let mut max_seen = 0;
    let mut transitions = 0;
    let mut prev = Status::Idle;
    for _ in 0..2000u32 {
        let status = sender.tick().unwrap();
        if status != prev {
            transitions += 1;
            prev = status;
        }
        if status == Status::Transmit(true) {
            assert_eq!(sender.pwm().duty, 333);
        } else {
            assert_eq!(sender.pwm().duty, 0);
        }
        max_seen = max_seen.max(sender.pwm().duty);
    }

    assert_eq!(max_seen, 333);
    // The pwm must only be written on state transitions
    assert_eq!(sender.pwm().writes, transitions);
}
