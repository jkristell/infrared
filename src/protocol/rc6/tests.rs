use crate::{
    Level,
    protocol::{rc6::Rc6Command, Rc6},
    receiver::{BufferReceiver, ProtocolDecoder},
    sender::PulseBuffer,
    Receiver,
};

#[test]
fn state_debug() {
    let cmd = Rc6Command::new(70, 20);

    const SAMPLE_RATE: u32 = 1_000_000;

    let mut sender = PulseBuffer::<96>::new();

    sender.load::<Rc6>(&cmd, SAMPLE_RATE);


    let mut level = Level::Space;

    let mut recv = Receiver::<Rc6>::new(SAMPLE_RATE);

    let mut res = None;

    for dist in sender.buffer() {
        level = level.toggle();

        let s0 = recv.decoder.state;
        let cmd = recv.event(*dist, level);

        println!(
            "{} ({}): {:?} -> {:?}",
            level.is_mark() as u32, dist, s0, recv.decoder.state
        );

        if let Ok(Some(cmd)) = cmd {
            res = Some(cmd);
        }
    }

    let res_cmd = res.unwrap();
    assert_eq!(res_cmd, cmd)
}

#[test]
#[rustfmt::skip]
fn decode_buffer() {
    let dists = [
        0, 106, 35, 17, 35, 17, 17, 17, 17, 17, 35, 35, 17, 35, 35, 17, 17, 17, 17, 35, 17, 17, 35,
        17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 35, 35, 35,
    ];

    let recv = BufferReceiver::<Rc6>::with_resolution(40_000);

    let iter = recv.iter(&dists);

    println!("{:?}", iter.decoder.spans());

    let cmds = iter.collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 1);

    for cmd in &cmds {
        assert_eq!(cmd.address, 70);
        assert_eq!(cmd.command, 2);
    }
}

#[test]
fn all_commands() {
    let mut ptb = PulseBuffer::<96>::new();
    const FREQ: u32 = 40_000;

    for address in 0..255 {
        for cmdnum in 0..255 {
            ptb.reset();

            let cmd = Rc6Command::new(address, cmdnum);
            ptb.load::<Rc6>(&cmd, FREQ);

            let recv = BufferReceiver::<Rc6>::with_resolution(FREQ);

            let cmdres = recv.iter(ptb.buffer()).next().unwrap();

            assert_eq!(cmd.address, cmdres.address);
            assert_eq!(cmd.command, cmdres.command);
        }
    }
}
