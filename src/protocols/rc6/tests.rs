use crate::protocols::rc6::Rc6Command;
use crate::protocols::Rc6;
use crate::recv::{BufferReceiver, EventReceiver};
use crate::send::{InfraredSender, PulsedataBuffer};

#[test]
fn newpulse() {
    let cmd = Rc6Command::new(70, 20);

    let sample_rate = 1_000_000;

    let mut sender = PulsedataBuffer::new();
    let state = Rc6::sender_state(sample_rate);

    sender.load::<Rc6>(&state, &cmd);

    let b = sender.buf;
    let len = sender.offset;

    let mut edge = false;
    let mut recv: EventReceiver<Rc6> = EventReceiver::new(sample_rate);

    let mut res_cmd = None;

    for dist in &b[..len] {
        edge = !edge;

        let s0 = recv.state.state;
        let cmd = recv.update(edge, *dist as u32);

        println!(
            "{} ({}): {:?} -> {:?}",
            edge as u32, dist, s0, recv.state.state
        );

        if let Ok(Some(cmd)) = cmd {
            res_cmd = Some(cmd);
        }
    }

    let res_cmd = res_cmd.unwrap();
    assert_eq!(res_cmd, cmd)
}

#[test]
#[rustfmt::skip]
fn basic() {
    let dists = [
        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34,
        19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,

        0, 106, 35, 17, 35, 17, 17, 17, 17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35,
        17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 35, 35, 35,

        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34,
        19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
    ];


    let recv = BufferReceiver::new(&dists, 40_000);
    let cmds = recv.iter::<Rc6>().collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 3);

    for cmd in &cmds {
        assert_eq!(cmd.addr, 70);
        assert_eq!(cmd.cmd, 2);
    }
}

#[test]
fn all_commands() {
    let mut ptb = PulsedataBuffer::new();
    let sample_rate = 40_000;
    let state = Rc6::sender_state(sample_rate);

    for address in 0..255 {
        for cmdnum in 0..255 {
            ptb.reset();

            let cmd = Rc6Command::new(address, cmdnum);
            ptb.load::<Rc6>(&state, &cmd);
            let brecv = BufferReceiver::new(&ptb.buf, sample_rate);
            let cmdres = brecv.iter::<Rc6>().next().unwrap();

            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}
