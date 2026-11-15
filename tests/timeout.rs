#![cfg(feature = "nec")]

use std::{cell::Cell, rc::Rc};

use infrared::{
    PolledReceiver, Receiver,
    protocol::{Nec, nec::NecCommand},
    receiver::{ClockedReceiver, FnClock, MultiReceiver},
    sender::PulseSender,
};

const RES: u32 = 1_000_000;

/// Edge distances in µs of an encoded NEC frame. The first edge is the start of an IR mark.
fn nec_frame(addr: u8, cmd: u8) -> Vec<u32> {
    let mut sender: PulseSender = PulseSender::new(RES);
    sender.load::<Nec>(&NecCommand {
        addr,
        cmd,
        repeat: false,
    });
    sender.buffer().to_vec()
}

/// Feed `dists` to `recv`, alternating edges starting with an IR mark
fn feed(recv: &mut Receiver<Nec>, dists: &[u32]) -> Vec<Result<Option<NecCommand>, ()>> {
    let mut edge = false;
    dists
        .iter()
        .map(|&dt| {
            edge = !edge;
            recv.event(dt, edge).map_err(|_| ())
        })
        .collect()
}

#[test]
fn idle_timeout_is_longer_than_any_symbol() {
    let recv: Receiver<Nec> = Receiver::new(RES);
    let idle = recv.idle_timeout();

    let longest = *nec_frame(0x12, 0x34).iter().max().unwrap();
    assert!(idle > longest, "{idle} <= {longest}");
    // But shorter than the ~40 ms gap between a NEC frame and its first repeat
    assert!(idle < 40_000, "{idle}");
}

#[test]
fn timeout_discards_partial_frame() {
    let frame = nec_frame(0x12, 0x34);
    let partial = &frame[..frame.len() / 2];

    // Without a timeout, the interrupted frame makes the next one report an error
    let mut recv: Receiver<Nec> = Receiver::new(RES);
    feed(&mut recv, partial);
    let mut next = vec![50_000];
    next.extend_from_slice(&frame[1..]);
    assert!(feed(&mut recv, &next).iter().any(Result::is_err));

    // With a timeout, the next frame decodes cleanly
    let mut recv: Receiver<Nec> = Receiver::new(RES);
    assert!(feed(&mut recv, partial).iter().all(Result::is_ok));
    assert_eq!(recv.timeout(), Ok(None));
    let res = feed(&mut recv, &next);
    assert!(res.iter().all(Result::is_ok));
    let cmds: Vec<_> = res.into_iter().flatten().flatten().collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x12, 0x34));
}

#[test]
fn timeout_when_idle_is_noop() {
    let frame = nec_frame(1, 2);
    let mut recv: Receiver<Nec> = Receiver::new(RES);

    assert_eq!(recv.timeout(), Ok(None));
    let cmds: Vec<_> = feed(&mut recv, &frame)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!(recv.timeout(), Ok(None));
}

#[cfg(feature = "rc5")]
#[test]
fn multireceiver_timeout() {
    use infrared::protocol::Rc5;

    let frame = nec_frame(0x12, 0x34);
    let mut recv: MultiReceiver<2, (Nec, Rc5)> = MultiReceiver::new(RES);
    // The longest idle timeout of the protocols is used
    assert_eq!(
        recv.idle_timeout(),
        Receiver::<Nec>::new(RES).idle_timeout()
    );

    let mut edge = false;
    for &dt in &frame[..frame.len() / 2] {
        edge = !edge;
        recv.event(dt, edge);
    }
    assert!(recv.timeout().iter().all(Option::is_none));

    let mut edge = false;
    let mut next = vec![50_000];
    next.extend_from_slice(&frame[1..]);
    let mut cmds = vec![];
    for &dt in &next {
        edge = !edge;
        cmds.extend(recv.event_iter(dt, edge));
    }
    assert_eq!(cmds.len(), 1);
}

/// Sample `dists` at 1 sample per tick into input levels (true = IR mark), ending idle
fn sample(dists: &[u32]) -> Vec<bool> {
    let mut level = false;
    let mut out = vec![];
    for &d in dists {
        out.extend(std::iter::repeat_n(level, d as usize));
        level = !level;
    }
    // Let the final edge be seen, then return to idle
    out.extend(std::iter::repeat_n(level, 20));
    out.extend(std::iter::repeat_n(false, 20));
    out
}

#[test]
fn polled_receiver_flushes_on_idle() {
    // 20 kHz sampling: 50 µs per sample
    const FREQ: u32 = 20_000;
    let frame: Vec<u32> = nec_frame(0x12, 0x34).iter().map(|d| d / 50).collect();

    let mut recv: PolledReceiver<Nec> = PolledReceiver::new(FREQ);
    let idle = recv.idle_timeout() as usize;

    // Half a frame, then idle long enough to flush, then a complete frame
    let mut samples = sample(&frame[..frame.len() / 2 + 1]);
    samples.extend(std::iter::repeat_n(false, idle + 10));
    samples.extend(sample(&frame));

    let res: Vec<_> = samples.iter().map(|&l| recv.poll(l)).collect();
    assert!(res.iter().all(Result::is_ok));
    let cmds: Vec<_> = res.into_iter().flatten().flatten().collect();
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x12, 0x34));
}

type TestClock = FnClock<Box<dyn FnMut() -> u32>, u32>;

fn test_clock() -> (Rc<Cell<u32>>, TestClock) {
    let time = Rc::new(Cell::new(0));
    let t = time.clone();
    let clock: TestClock = FnClock::new(RES, Box::new(move || t.get()));
    (time, clock)
}

#[test]
fn clocked_receiver_decodes() {
    let (time, clock) = test_clock();
    let mut recv = infrared::receiver().nec().build_clocked(clock);

    let mut edge = false;
    let mut cmds = vec![];
    for &dt in &nec_frame(0x56, 0x78) {
        time.set(time.get() + dt);
        edge = !edge;
        if let Some(cmd) = recv.event(edge).unwrap() {
            cmds.push(cmd);
        }
    }
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x56, 0x78));
}

#[test]
fn clocked_receiver_timeout_checks_elapsed_time() {
    let (time, clock) = test_clock();
    let mut recv: ClockedReceiver<Receiver<Nec>, _> = ClockedReceiver::with_clock(clock);
    let frame = nec_frame(0x12, 0x34);
    let (first, second) = frame.split_at(frame.len() / 2);

    let edge = Cell::new(false);
    let feed = |recv: &mut ClockedReceiver<Receiver<Nec>, TestClock>, dists: &[u32]| {
        let mut out = vec![];
        for &dt in dists {
            time.set(time.get() + dt);
            edge.set(!edge.get());
            out.push(recv.event(edge.get()));
        }
        out
    };

    // A timeout() call in the middle of a frame must not disturb it
    feed(&mut recv, first);
    assert_eq!(recv.timeout(), Ok(None));
    let cmds: Vec<_> = feed(&mut recv, second)
        .into_iter()
        .flatten()
        .flatten()
        .collect();
    assert_eq!(cmds.len(), 1);

    // An interrupted frame is discarded once the idle timeout has passed
    feed(&mut recv, first);
    time.set(time.get() + recv.idle_timeout());
    assert_eq!(recv.timeout(), Ok(None));
    let mut next = vec![50_000];
    next.extend_from_slice(&frame[1..]);
    if edge.get() {
        // Restart with an IR mark
        next.insert(0, 1);
    }
    let res = feed(&mut recv, &next);
    assert!(res.iter().all(Result::is_ok));
    assert_eq!(res.into_iter().flatten().flatten().count(), 1);
}

#[test]
fn clocked_receiver_custom_idle_timeout() {
    let (_time, clock) = test_clock();
    let recv: ClockedReceiver<Receiver<Nec>, _> =
        ClockedReceiver::with_clock(clock).with_idle_timeout(100_000);
    assert_eq!(recv.idle_timeout(), 100_000);
}

#[test]
fn clocked_receiver_poll() {
    let (time, clock) = test_clock();
    let mut recv: ClockedReceiver<Receiver<Nec>, _> = ClockedReceiver::with_clock(clock);

    // Levels sampled every 50 µs; poll() picks out the edges itself
    let samples = sample(
        &nec_frame(0x9a, 0xbc)
            .iter()
            .map(|d| d / 50)
            .collect::<Vec<_>>(),
    );
    let mut cmds = vec![];
    for level in samples {
        time.set(time.get() + 50);
        if let Some(cmd) = recv.poll(level).unwrap() {
            cmds.push(cmd);
        }
    }
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x9a, 0xbc));
}

#[cfg(feature = "embedded-hal")]
#[test]
fn clocked_pin_receiver() {
    use std::convert::Infallible;

    use embedded_hal::digital::{ErrorType, InputPin};

    struct Pin(Rc<Cell<bool>>);
    impl ErrorType for Pin {
        type Error = Infallible;
    }
    impl InputPin for Pin {
        fn is_high(&mut self) -> Result<bool, Infallible> {
            Ok(self.0.get())
        }
        fn is_low(&mut self) -> Result<bool, Infallible> {
            Ok(!self.0.get())
        }
    }

    let (time, clock) = test_clock();
    let level = Rc::new(Cell::new(true));
    let mut recv = infrared::receiver()
        .nec()
        .build_clocked_with_pin(clock, Pin(level.clone()));

    let mut cmds = vec![];
    for &dt in &nec_frame(0x11, 0x22) {
        time.set(time.get() + dt);
        // Active low: IR mark pulls the pin low
        level.set(!level.get());
        if let Some(cmd) = recv.event().unwrap() {
            cmds.push(cmd);
        }
    }
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x11, 0x22));

    time.set(time.get() + 100_000);
    assert_eq!(recv.timeout(), Ok(None));
}

#[cfg(feature = "rc5")]
#[test]
fn clocked_multireceiver() {
    use infrared::{ProtocolId, protocol::Rc5};

    let (time, clock) = test_clock();
    let mut recv: ClockedReceiver<MultiReceiver<2, (Nec, Rc5)>, _> =
        ClockedReceiver::with_clock_multi(clock);

    let mut edge = false;
    let mut cmds = vec![];
    for &dt in &nec_frame(1, 2) {
        time.set(time.get() + dt);
        edge = !edge;
        cmds.extend(recv.event_iter(edge));
    }
    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].protocol, ProtocolId::Nec);
}

#[test]
fn clocked_receiver_handles_clock_wraparound() {
    let (time, clock) = test_clock();
    let frame = nec_frame(0x12, 0x34);
    let half: u32 = frame[..frame.len() / 2].iter().sum();
    // Start so that the counter wraps in the middle of the first frame
    time.set(u32::MAX - half);
    let mut recv: ClockedReceiver<Receiver<Nec>, _> = ClockedReceiver::with_clock(clock);

    let mut edge = false;
    let mut cmds = vec![];
    for &dt in &frame {
        time.set(time.get().wrapping_add(dt));
        edge = !edge;
        cmds.extend(recv.event(edge).unwrap());
    }
    assert_eq!(cmds.len(), 1);
    assert_eq!((cmds[0].addr, cmds[0].cmd), (0x12, 0x34));

    // An interrupted frame is discarded across the wrap as well
    time.set(u32::MAX - half);
    for &dt in &frame[..frame.len() / 2] {
        time.set(time.get().wrapping_add(dt));
        edge = !edge;
        recv.event(edge).unwrap();
    }
    time.set(time.get().wrapping_add(recv.idle_timeout()));
    recv.timeout().unwrap();
    let mut next = vec![50_000];
    next.extend_from_slice(&frame[1..]);
    if edge {
        next.insert(0, 1);
    }
    let mut res = vec![];
    for &dt in &next {
        time.set(time.get().wrapping_add(dt));
        edge = !edge;
        res.push(recv.event(edge));
    }
    assert!(res.iter().all(Result::is_ok));
    assert_eq!(res.into_iter().flatten().flatten().count(), 1);
}
