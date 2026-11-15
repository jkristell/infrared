use crate::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::BufferReceiver,
    remotecontrol::{Action, Button},
    sender::PulseBuffer,
};

#[test]
fn rc5_command() {
    let cmd = Rc5Command::new(20, 15, false);
    assert_eq!(cmd, Rc5Command::unpack(cmd.pack()))
}

#[test]
fn test_bufrecv() {
    let dists = [
        0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37, 0, 37, 34,
        72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
    ];

    let r = BufferReceiver::<Rc5>::with_resolution(40_000);

    let v: std::vec::Vec<_> = r.iter(&dists).collect();
    assert_eq!(v.len(), 2);

    for c in &v {
        assert_eq!(c.address, 20);
        assert_eq!(c.command, 9);
    }
}

#[test]
#[rustfmt::skip]
fn command_mixed() {

    use crate::remotecontrol::rc5;

    let dists = [
        57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36, 71, 36,
        26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36, 71, 36,
        // From another rc5 like protocol but not standard rc5, should be ignored by the receiver
        10254, 37, 35, 37, 34, 37, 35, 37, 35, 73, 34, 38, 70, 37, 141, 38, 34, 37, 35, 37, 34, 38, 34, 37, 70, 73, 35, 37, 35, 37, 34, 38, 34, 37, 34, 38,
        50973, 38, 34, 73, 70, 73, 70, 74, 34, 37, 35, 37, 34, 38, 34, 37, 35, 37, 34, 38, 70, 37,
    ];

    let r = BufferReceiver::<Rc5>::with_resolution(40_000);
    let v: std::vec::Vec<_> = r.iter( &dists).collect();
    assert_eq!(v.len(), 3);

    for c in &v {
        assert_eq!(c.address, 20);
        assert_eq!(c.command, 1);
    }

    let r = BufferReceiver::<Rc5, u32, Button< rc5::CdPlayer>>::with_resolution(40_000);

    let v: std::vec::Vec<_> = r.iter( &dists).collect();
    assert_eq!(v.len(), 3);

    for c in &v {
        assert_eq!(c.action(), Some(Action::One));
    }

}

#[test]
fn all_commands() {
    const SAMPLERATE: u32 = 40_000;

    let mut ptb = PulseBuffer::<96>::new();

    for address in 0..32 {
        for cmdnum in 0..64 {
            ptb.reset();

            let cmd: Rc5Command = Rc5Command::new(address, cmdnum, false);
            ptb.load::<Rc5>(&cmd, SAMPLERATE);

            let r = BufferReceiver::<Rc5>::with_resolution(SAMPLERATE);

            let cmdres = r.iter(ptb.buffer()).next().unwrap();

            assert_eq!(cmd.address, cmdres.address);
            assert_eq!(cmd.command, cmdres.command);
        }
    }
}

#[test]
fn timer_resolution() {
    test_freq::<20_000>();
    test_freq::<40_000>();
    test_freq::<48_000_000>();
}

fn test_freq<const F: u32>() {
    let mut ptb = PulseBuffer::<96>::new();
    let cmd: Rc5Command = Rc5Command::new(10, 2, false);
    ptb.load::<Rc5>(&cmd, F);
    //let mut r = Builder::new().rc5().resolution(F).buffer(&ptb.buf).build();
    let r = BufferReceiver::<Rc5>::with_resolution(F);

    let cmdres = r.iter(ptb.buffer()).next().unwrap();
    assert_eq!(cmd.address, cmdres.address);
    assert_eq!(cmd.command, cmdres.command);
}

#[cfg(feature = "fugit")]
#[test]
fn rc5_fugit_u64() {
    use fugit::TimerInstantU64;

    use crate::receiver::Receiver;

    // Rc5 command, address 20, command 9, captured at 40 kHz (25 µs ticks)
    const DISTS: [u64; 20] = [
        0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
    ];

    let mut recv: Receiver<Rc5, TimerInstantU64<1_000_000>> = Receiver::with_fugit64();

    let mut cmds = std::vec::Vec::new();
    let mut t = 0;
    for (i, dt) in DISTS.iter().enumerate() {
        let edge = i % 2 == 0;
        t += dt * 25;
        if let Ok(Some(cmd)) = recv.event_instant(TimerInstantU64::from_ticks(t), edge) {
            cmds.push(cmd);
        }
    }

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].address, 20);
    assert_eq!(cmds[0].command, 9);
}
