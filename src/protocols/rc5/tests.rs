use crate::protocols::rc5::Rc5Command;
use crate::recv::BufferReceiver;
use crate::protocols::Rc5;
use crate::send::PulsedataBuffer;

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

    let r: BufferReceiver<Rc5> = BufferReceiver::with_values(&dists, 40_000);
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
    let dists = [
        57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36, 71, 36,
        26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36, 71, 36,
        // From another rc5 like protocol but not standard rc5, should be ignored by the receiver
        10254, 37, 35, 37, 34, 37, 35, 37, 35, 73, 34, 38, 70, 37, 141, 38, 34, 37, 35, 37, 34, 38, 34, 37, 70, 73, 35, 37, 35, 37, 34, 38, 34, 37, 34, 38,
        50973, 38, 34, 73, 70, 73, 70, 74, 34, 37, 35, 37, 34, 38, 34, 37, 35, 37, 34, 38, 70, 37,
    ];

    let r: BufferReceiver<Rc5> = BufferReceiver::with_values(&dists, 40_000);
    let v: std::vec::Vec<_> = r.iter().collect();
    assert_eq!(v.len(), 3);

    for c in &v {
        assert_eq!(c.addr, 20);
        assert_eq!(c.cmd, 1);
    }
}

#[test]
fn all_commands() {
    let mut ptb = PulsedataBuffer::with_samplerate(40_000);

    for address in 0..32 {
        for cmdnum in 0..64 {
            let cmd: Rc5Command = Rc5Command::new(address, cmdnum, false);
            ptb.load(&cmd);
            let brecv: BufferReceiver<Rc5> = BufferReceiver::with_values(&ptb.buf, 40_000);
            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}
