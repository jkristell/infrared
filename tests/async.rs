#![cfg(all(feature = "async", feature = "nec"))]

use std::{
    cell::RefCell,
    collections::VecDeque,
    convert::Infallible,
    future::{poll_fn, Future},
    pin::pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use embedded_hal_async::{delay::DelayNs, digital::Wait};
use infrared::{
    protocol::{nec::NecCommand, Nec},
    receiver::{AsyncReceiver, FnClock},
    sender::TickSender,
    Receiver,
};

const RES: u32 = 1_000_000;
/// Simulated time advanced per executor poll, in µs
const STEP: u32 = 5;

/// Simulated world: the current time and the scheduled input edges
#[derive(Default)]
struct Sim {
    now: u32,
    edges: VecDeque<u32>,
}

type SharedSim = Rc<RefCell<Sim>>;

impl Sim {
    /// Schedule the edges of a pulse train given as edge distances, starting with an IR mark
    fn schedule(&mut self, dists: &[u32]) {
        let mut t = self.edges.back().copied().unwrap_or(self.now);
        for &d in dists {
            t += d;
            self.edges.push_back(t);
        }
    }
}

struct SimPin(SharedSim);

impl embedded_hal::digital::ErrorType for SimPin {
    type Error = Infallible;
}

impl SimPin {
    async fn next_edge(&mut self) -> Result<(), Infallible> {
        poll_fn(|_| {
            let mut sim = self.0.borrow_mut();
            match sim.edges.front() {
                Some(&t) if t <= sim.now => {
                    sim.edges.pop_front();
                    Poll::Ready(Ok(()))
                }
                _ => Poll::Pending,
            }
        })
        .await
    }
}

impl Wait for SimPin {
    async fn wait_for_high(&mut self) -> Result<(), Infallible> {
        self.next_edge().await
    }
    async fn wait_for_low(&mut self) -> Result<(), Infallible> {
        self.next_edge().await
    }
    async fn wait_for_rising_edge(&mut self) -> Result<(), Infallible> {
        unimplemented!()
    }
    async fn wait_for_falling_edge(&mut self) -> Result<(), Infallible> {
        unimplemented!()
    }
    async fn wait_for_any_edge(&mut self) -> Result<(), Infallible> {
        unimplemented!()
    }
}

struct SimDelay(SharedSim);

impl DelayNs for SimDelay {
    async fn delay_ns(&mut self, ns: u32) {
        let deadline = self.0.borrow().now + ns / 1000;
        poll_fn(|_| {
            if self.0.borrow().now >= deadline {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
        .await
    }
}

/// Run `fut`, advancing the simulated time by `STEP` between polls
fn run<F: Future>(sim: &SharedSim, fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(out) = fut.as_mut().poll(&mut cx) {
            return out;
        }
        let mut sim = sim.borrow_mut();
        assert!(sim.now < 10_000_000, "simulation timed out");
        sim.now += STEP;
    }
}

type SimClock = FnClock<Box<dyn FnMut() -> u32>, u32>;

fn setup() -> (SharedSim, AsyncReceiver<Receiver<Nec>, SimClock, SimPin, SimDelay>) {
    let sim = SharedSim::default();
    let s = sim.clone();
    let clock: SimClock = FnClock::new(RES, Box::new(move || s.borrow().now));
    let recv = infrared::receiver().nec().build_async(
        clock,
        SimPin(sim.clone()),
        SimDelay(sim.clone()),
    );
    (sim, recv)
}

fn nec_frame(address: u8, command: u8) -> Vec<u32> {
    let mut sender: TickSender = TickSender::new(RES);
    sender.load::<Nec>(&NecCommand {
        address,
        command,
        repeat: false,
    });
    sender.buffer().to_vec()
}

#[test]
fn receive() {
    let (sim, mut recv) = setup();
    sim.borrow_mut().schedule(&[1000]);
    sim.borrow_mut().schedule(&nec_frame(0x12, 0x34)[1..]);

    let cmd = run(&sim, recv.receive()).unwrap();
    assert_eq!((cmd.address, cmd.command), (0x12, 0x34));
}

#[test]
fn interrupted_frame_is_discarded_on_idle() {
    let (sim, mut recv) = setup();
    let frame = nec_frame(0x12, 0x34);

    // Half a frame, ending with an IR mark, then 50 ms of silence before a complete frame
    let mut partial = frame[..frame.len() / 2].to_vec();
    if partial.len() % 2 == 0 {
        partial.pop();
    }
    partial[0] = 1000;
    sim.borrow_mut().schedule(&partial);
    // The interrupted mark ends, then the next frame starts
    sim.borrow_mut().schedule(&[560, 50_000]);
    sim.borrow_mut().schedule(&frame[1..]);

    let mut results = vec![];
    let cmd = loop {
        let res = run(&sim, recv.next());
        results.push(res);
        if let Ok(Some(cmd)) = res {
            break cmd;
        }
    };
    assert!(results.iter().all(Result::is_ok), "{results:?}");
    assert_eq!((cmd.address, cmd.command), (0x12, 0x34));
}

#[test]
fn back_to_back_frames() {
    let (sim, mut recv) = setup();
    sim.borrow_mut().schedule(&[1000]);
    sim.borrow_mut().schedule(&nec_frame(1, 2)[1..]);
    // The next frame follows 40 ms after the stop mark
    sim.borrow_mut().schedule(&[40_000]);
    sim.borrow_mut().schedule(&nec_frame(3, 4)[1..]);

    let first = run(&sim, recv.receive()).unwrap();
    let second = run(&sim, recv.receive()).unwrap();
    assert_eq!((first.address, first.command), (1, 2));
    assert_eq!((second.address, second.command), (3, 4));
}

#[cfg(feature = "rc5")]
#[test]
fn multi_receive() {
    use infrared::{
        protocol::Rc5,
        receiver::MultiReceiver,
        ProtocolId,
    };

    let sim = SharedSim::default();
    let s = sim.clone();
    let clock: SimClock = FnClock::new(RES, Box::new(move || s.borrow().now));
    let mut recv: AsyncReceiver<MultiReceiver<2, (Rc5, Nec)>, _, _, _> =
        AsyncReceiver::with_clock_multi(clock, SimPin(sim.clone()), SimDelay(sim.clone()));

    sim.borrow_mut().schedule(&[1000]);
    sim.borrow_mut().schedule(&nec_frame(5, 6)[1..]);
    let cmd = run(&sim, recv.receive()).unwrap();
    assert_eq!(cmd.protocol, ProtocolId::Nec);
}

/// Records every duty cycle change with the simulated time
struct RecordingPwm {
    sim: SharedSim,
    log: Vec<(u32, u16)>,
}

impl embedded_hal::pwm::ErrorType for RecordingPwm {
    type Error = Infallible;
}

impl embedded_hal::pwm::SetDutyCycle for RecordingPwm {
    fn max_duty_cycle(&self) -> u16 {
        1000
    }

    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Infallible> {
        self.log.push((self.sim.borrow().now, duty));
        Ok(())
    }
}

#[test]
fn async_sender() {
    use infrared::{receiver::BufferReceiver, sender::AsyncSender};

    let sim = SharedSim::default();
    let pwm = RecordingPwm { sim: sim.clone(), log: vec![] };
    let mut sender = AsyncSender::new(pwm, SimDelay(sim.clone()));
    // Initially off
    assert_eq!(sender.pwm().log, [(0, 0)]);

    let cmd = NecCommand { address: 0x12, command: 0x34, repeat: false };
    run(&sim, sender.send::<Nec>(&cmd)).unwrap();

    let log = &sender.pwm().log[1..];
    // Alternating 25% duty marks and spaces, ending with the carrier off
    assert!(log.iter().enumerate().all(|(i, &(_, d))| d == if i % 2 == 0 { 250 } else { 0 }));
    assert_eq!(log.len() % 2, 0);

    // The recorded edges decode back to the command
    let mut dists = vec![0];
    dists.extend(log.windows(2).map(|w| w[1].0 - w[0].0));
    let cmds: Vec<_> = BufferReceiver::<Nec>::with_resolution(RES).iter(&dists).collect();
    assert_eq!(cmds, [cmd]);
}

#[test]
fn async_sender_cancelled_turns_carrier_off() {
    use infrared::sender::AsyncSender;

    let sim = SharedSim::default();
    let pwm = RecordingPwm { sim: sim.clone(), log: vec![] };
    let mut sender = AsyncSender::new(pwm, SimDelay(sim.clone()));

    let cmd = NecCommand { address: 1, command: 2, repeat: false };
    {
        // Start sending, then drop the future in the middle of the header mark
        let mut fut = pin!(sender.send::<Nec>(&cmd));
        let mut cx = Context::from_waker(Waker::noop());
        assert!(fut.as_mut().poll(&mut cx).is_pending());
    }
    assert_eq!(sender.pwm().log.last(), Some(&(0, 0)));
    assert_eq!(sender.pwm().log.len(), 3);
}
