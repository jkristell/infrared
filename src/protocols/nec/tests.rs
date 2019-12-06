use crate::{
    nec::{NecCommand, NecVariant, Nec, Nec16Variant, NecStandard, SamsungVariant},
    ReceiverState, ReceiverStateMachine,
};

#[test]
fn standard_nec() {
    let dists = [
        0, 363, 177, 24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21, 24, 66, 24,
        66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 24, 21,
        24, 21, 24, 21, 24, 21, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66, 24,
    ];

    let mut recv = Nec::new(40_000);
    let mut edge = false;
    let mut tot = 0;
    let mut state = ReceiverState::Idle;

    for dist in dists.iter() {
        edge = !edge;
        tot += *dist;
        state = recv.event(edge, tot);
    }

    if let ReceiverState::Done(cmd) = state {
        assert_eq!(cmd.addr, 0);
        assert_eq!(cmd.cmd, 12);
    } else {
        assert!(false);
    }
}

#[test]
fn cmd_standard() {
    let cmd = NecCommand::new(7, 44);
    let bits = NecStandard::encode_command(cmd);

    assert!(NecStandard::verify_command(bits));

    assert_eq!(bits, 0xD32CF807);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (!bits & 0xFF));

    let cmd2 = NecStandard::decode_command(bits);
    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_samsumg() {
    let cmd = NecCommand::new(7, 44);
    let bits = SamsungVariant::encode_command(cmd);

    assert!(SamsungVariant::verify_command(bits));

    assert_eq!(bits, 0xD32C0707);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (bits & 0xFF));

    let cmd2 = SamsungVariant::decode_command(bits);
    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_nec16() {
    let cmd = NecCommand::new(28114, 220);
    let bits = Nec16Variant::encode_command(cmd);

    assert!(Nec16Variant::verify_command(bits));

    assert_eq!(bits, 0x23DC6DD2);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));

    let cmd2 = Nec16Variant::decode_command(bits);
    assert_eq!(cmd, cmd2);
}
