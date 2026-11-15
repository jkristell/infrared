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
    sender::PulseSender,
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

fn nec_frame(addr: u8, cmd: u8) -> Vec<u32> {
    let mut sender: PulseSender<128> = PulseSender::new();
    sender.load_command::<Nec, RES>(&NecCommand {
        addr,
        cmd,
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
    assert_eq!((cmd.addr, cmd.cmd), (0x12, 0x34));
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
    assert_eq!((cmd.addr, cmd.cmd), (0x12, 0x34));
}

#[test]
fn back_to_back_frames() {
    let (sim, mut recv) = setup();
    sim.borrow_mut().schedule(&[1000]);
    sim.borrow_mut().schedule(&nec_frame(1, 2)[1..]);
    // The stop bit mark ends, and the next frame follows 40 ms later
    sim.borrow_mut().schedule(&[560, 40_000]);
    sim.borrow_mut().schedule(&nec_frame(3, 4)[1..]);

    let first = run(&sim, recv.receive()).unwrap();
    let second = run(&sim, recv.receive()).unwrap();
    assert_eq!((first.addr, first.cmd), (1, 2));
    assert_eq!((second.addr, second.cmd), (3, 4));
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
