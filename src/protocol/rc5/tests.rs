use crate::{
    protocol::{rc5::Rc5Command, Rc5},
    receiver::Builder,
    remotecontrol::Action,
    sender::PulsedataBuffer,
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

    let mut r = Builder::new()
        .rc5()
        .resolution(40_000)
        .buffer(&dists)
        .build();
    let v: std::vec::Vec<_> = r.iter().collect();
    assert_eq!(v.len(), 2);

    for c in &v {
        assert_eq!(c.addr, 20);
        assert_eq!(c.cmd, 9);
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

    let mut r = Builder::new()
        .rc5()
        .resolution(40_000).buffer(&dists).build();
    let v: std::vec::Vec<_> = r.iter().collect();
    assert_eq!(v.len(), 3);

    for c in &v {
        println!("{:?}", c);
        assert_eq!(c.addr, 20);
        assert_eq!(c.cmd, 1);
    }

    let mut r = crate::Receiver::builder()
        .rc5()
        .remotecontrol(rc5::CdPlayer)
        .resolution(40_000).buffer(&dists).build();

    let v: std::vec::Vec<_> = r.iter().collect();
    assert_eq!(v.len(), 3);

    for c in &v {
        assert_eq!(c.action(), Some(Action::One));
    }

}

#[test]
fn all_commands() {
    const SAMPLERATE: u32 = 40_000;

    let mut ptb = PulsedataBuffer::<96>::new();

    for address in 0..32 {
        for cmdnum in 0..64 {
            ptb.reset();

            let cmd: Rc5Command = Rc5Command::new(address, cmdnum, false);
            ptb.load::<Rc5, SAMPLERATE>(&cmd);
            let mut r = Builder::new()
                .rc5()
                .resolution(SAMPLERATE)
                .buffer(&ptb.buf)
                .build();

            let cmdres = r.iter().next().unwrap();

            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
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
    let mut ptb = PulsedataBuffer::<96>::new();
    let cmd: Rc5Command = Rc5Command::new(10, 2, false);
    ptb.load::<Rc5, F>(&cmd);
    let mut r = Builder::new().rc5().resolution(F).buffer(&ptb.buf).build();

    let cmdres = r.iter().next().unwrap();
    assert_eq!(cmd.addr, cmdres.addr);
    assert_eq!(cmd.cmd, cmdres.cmd);
}
