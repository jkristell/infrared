use crate::{
    protocol::{rc6::Rc6Command, Rc6},
    receiver::BufferInputReceiver,
    sender::PulsedataBuffer,
};

/*
#[test]
fn newpulse() {
    let cmd = Rc6Command::new(70, 20);

    const SAMPLE_RATE: u32 = 1_000_000;

    let mut sender = PulsedataBuffer::<96>::new();

    sender.load::<Rc6, SAMPLE_RATE>(&cmd);

    let b = sender.buf;
    let len = sender.offset;

    let mut edge = false;

    let mut recv = BufferInputReceiver::with_resolution(SAMPLE_RATE);

    let mut res_cmd = None;

    for dist in &b[..len] {
        edge = !edge;

        let s0 = recv.state.state;
        let cmd = recv.event(*dist, edge);

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

 */

#[test]
#[rustfmt::skip]
fn basic() {
    let dists = [
        /*
        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34,
        19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
        */

        0, 106, 35, 17, 35, 17, 17, 17, 17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35,
        17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 35, 35, 35,
/*
        0, 108, 34, 19, 34, 19, 16, 20, 16, 19, 34, 36, 16, 37, 34, 20, 16, 19, 16, 37, 17, 19, 34,
        19, 17, 19, 16, 19, 17, 19, 16, 20, 16, 19, 16, 37, 34, 20,
         */

    ];

    let mut recv = BufferInputReceiver::<Rc6>::with_resolution(40_000);

    let cmds = recv.iter(&dists).collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 1);

    for cmd in &cmds {
        assert_eq!(cmd.addr, 70);
        assert_eq!(cmd.cmd, 2);
    }
}

#[test]
fn all_commands() {
    let mut ptb = PulsedataBuffer::<96>::new();
    const SAMPLE_RATE: u32 = 40_000;

    for address in 0..255 {
        for cmdnum in 0..255 {
            ptb.reset();

            let cmd = Rc6Command::new(address, cmdnum);
            ptb.load::<Rc6, SAMPLE_RATE>(&cmd);

            let mut recv = BufferInputReceiver::<Rc6>::with_resolution(SAMPLE_RATE);

            let cmdres = recv.iter(&ptb.buf).next().unwrap();

            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}
