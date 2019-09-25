use crate::protocols::nec::{NecReceiver, SamsungType, StandardType};
use crate::prelude::*;
use crate::nec::{NecCommand, NecTypeTrait};

#[test]
fn standard_nec() {
    let dists = [0, 363, 177,
                 24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21,
                 24, 66, 24, 66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65,
                 24, 21, 24, 21, 24, 66, 24, 65, 24, 21, 24, 21, 24, 21, 24, 21,
                 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66,
                 24];

    let mut recv = NecReceiver::new(40_000);
    let mut edge = false;
    let mut tot = 0;
    let mut state = ReceiverState::Idle;

    for dist in dists.iter() {
        edge = !edge;
        tot += dist;
        state = recv.sample_edge(edge, tot);
    }

    if let ReceiverState::Done(cmd) = state {
        assert_eq!(cmd.addr, 0);
        assert_eq!(cmd.cmd, 12);
    } else {
        assert!(false);
    }
}

#[test]
fn cmd_encode() {

    let cmd = NecCommand {
        addr: 7,
        cmd: 44,
    };

    let standard = StandardType::encode_command(cmd);
    let samsung = SamsungType::encode_command(cmd);

    assert_eq!(standard, 0xD32CF807);
    assert_eq!((standard >> 24) & 0xFF, (!(standard >> 16) & 0xFF));
    assert_eq!((standard >> 8) & 0xFF, (!standard & 0xFF));

    assert_eq!(samsung, 0xD32C0707);
    assert_eq!((samsung >> 24) & 0xFF, (!(standard >> 16) & 0xFF));
    assert_eq!((samsung >> 8) & 0xFF, (standard & 0xFF));

    println!("{:X?}", samsung);
}