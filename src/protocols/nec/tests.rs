use crate::{
    protocols::nec::{Nec, Nec16, NecCommand, NecSamsung, NecStandard, NecVariant},
    recv::EventReceiver,
    BufferedReceiver,
};

#[test]
fn standard_nec() {
    use std::vec::Vec;

    let dists = [
        0, 363, 177, 24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21, 24, 66, 24,
        66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 24, 21,
        24, 21, 24, 21, 24, 21, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66, 24,
        0, 363, 177, 24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21, 24, 66, 24,
        66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 24, 21,
        24, 21, 24, 21, 24, 21, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66, 24,
    ];

    let mut recv: EventReceiver<Nec> = EventReceiver::new(40_000);

    let mut edge = false;
    let mut tot = 0;
    //let mut state = State::Idle;

    {
        let brecv: BufferedReceiver<Nec> = BufferedReceiver::new(&dists, 40_000);
        let cmds = brecv.collect::<Vec<_>>();

        assert_eq!(cmds.len(), 2);

        for cmd in &cmds {
            assert_eq!(cmd.addr, 0);
            assert_eq!(cmd.cmd, 12);
        }
    }

    for dist in dists.iter() {
        edge = !edge;
        tot += *dist;
        let cmd = recv.edge_event(edge, tot);

        if let Ok(Some(cmd)) = cmd {
            assert_eq!(cmd.addr, 0);
            assert_eq!(cmd.cmd, 12);
        }
    }
}

#[test]
fn cmd_standard() {
    let cmd = NecCommand::new(7, 44);
    let bits = NecStandard::cmd_to_bits(&cmd);

    assert!(NecStandard::cmd_is_valid(bits));

    assert_eq!(bits, 0xD32CF807);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (!bits & 0xFF));

    let cmd2 = NecStandard::cmd_from_bits(bits);
    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_samsumg() {
    let cmd = NecCommand::new(7, 44);
    let bits = NecSamsung::cmd_to_bits(&cmd);

    assert!(NecSamsung::cmd_is_valid(bits));

    assert_eq!(bits, 0xD32C0707);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (bits & 0xFF));

    let cmd2 = NecSamsung::cmd_from_bits(bits);
    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_nec16() {
    let cmd = NecCommand::new(28114, 220);
    let bits = Nec16::cmd_to_bits(&cmd);

    assert!(Nec16::cmd_is_valid(bits));

    assert_eq!(bits, 0x23DC6DD2);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));

    let cmd2 = Nec16::cmd_from_bits(bits);
    assert_eq!(cmd, cmd2);
}
