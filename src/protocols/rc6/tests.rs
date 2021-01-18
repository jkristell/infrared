use crate::protocols::rc6::{Rc6, Rc6Command};
use crate::recv::*;
use crate::bufrecv::BufferReceiver;
use crate::sender::PulseBuffer;
use crate::PulseLengths;

#[test]
fn pulses() {
    let mut recv: BufferReceiver<Rc6> = BufferReceiver::with_values(&[], 1_000_000);
    let cmds = [
        Rc6Command::new(72, 17),
        Rc6Command::new(0, 21),
        Rc6Command::new(1, 33),
    ];

    for cmd in &cmds {
        recv.add_cmd(cmd);
    }

    cmds.iter()
        .zip(recv.into_iter())
        .for_each(|(cmd, cmdrecv)| assert_eq!(cmd, &cmdrecv));
}

#[test]
fn newpulse() {
    let cmd = Rc6Command::new(70, 20);
    let mut b = [0u16; 96];
    let len = cmd.encode(&mut b);

    let mut edge = false;
    let mut recv: EventReceiver<Rc6> = EventReceiver::new(1_000_000);

    let mut res_cmd = None;

    for dist in &b[..len] {
        edge = !edge;

        let s0 = recv.sm.state;
        let cmd = recv.edge_event(edge, *dist as u32);

        println!(
            "{} ({}): {:?} -> {:?}",
            edge as u32, dist, s0, recv.sm.state
        );

        if let Ok(Some(cmd)) = cmd {
            res_cmd = Some(cmd);
        }
    }

    if let Some(res) = res_cmd {
        assert_eq!(res, cmd)
    }
}

#[test]
fn basic() {
    let dists = [
        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34,
        19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20, 0, 106, 35, 17, 35, 17, 17, 17,
        17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35, 17, 17, 17, 17, 17, 17, 17, 17,
        17, 17, 17, 17, 35, 35, 35, 0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20,
        16, 19, 16, 37, 17, 19, 34, 19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
    ];

    let recv: BufferReceiver<Rc6> = BufferReceiver::with_values(&dists, 40_000);

    let cmds = recv.iter().collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 3);

    for cmd in &cmds {
        assert_eq!(cmd.addr, 70);
        assert_eq!(cmd.cmd, 2);
    }
}

#[test]
fn all_commands() {
    let mut ptb = PulseBuffer::with_samplerate(40_000);

    for address in 0..255 {
        for cmdnum in 0..255 {
            let cmd: Rc6Command = Rc6Command::new(address, cmdnum);
            ptb.load(&cmd);
            let brecv: BufferReceiver<Rc6> = BufferReceiver::with_values(&ptb.buf, 40_000);
            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

