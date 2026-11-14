#![cfg(all(feature = "rc5", feature = "nec"))]

use infrared::{
    protocol::{Nec, Rc5},
    receiver::MultiReceiver,
    ProtocolId,
};

// Rc5 command, address 20, command 9, captured at 40 kHz
const DISTS: [u32; 20] = [
    0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
];

#[test]
fn multireceiver_decodes_rc5() {
    let mut mr: MultiReceiver<2, (Rc5, Nec)> = MultiReceiver::new(40_000);

    let mut cmds = Vec::new();
    for (i, dt) in DISTS.iter().enumerate() {
        let edge = i % 2 == 0;
        cmds.extend(mr.event_iter(*dt, edge));
    }

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].protocol, ProtocolId::Rc5);
    assert_eq!(cmds[0].address, 20);
    assert_eq!(cmds[0].command, 9);
}

#[test]
fn multireceiver_event_instant() {
    let mut mr: MultiReceiver<2, (Rc5, Nec)> = MultiReceiver::new(40_000);

    let mut cmds = Vec::new();
    let mut t = 0u32;
    for (i, dt) in DISTS.iter().enumerate() {
        let edge = i % 2 == 0;
        t += dt;
        cmds.extend(mr.event_instant(t, edge).into_iter().flatten());
    }

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].protocol, ProtocolId::Rc5);
    assert_eq!(cmds[0].address, 20);
    assert_eq!(cmds[0].command, 9);
}
