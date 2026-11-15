#![cfg(all(feature = "rc5", feature = "nec"))]

use infrared::{
    ProtocolId,
    protocol::{Nec, Rc5},
    receiver::MultiReceiver,
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

#[test]
fn message_from_command() {
    use infrared::{Message, protocol::rc5::Rc5Command};

    let msg = Message::from(Rc5Command::new(20, 9, true));
    assert_eq!(
        msg,
        Message {
            protocol: ProtocolId::Rc5,
            address: 20,
            command: 9,
            repeat: true,
            // Start bits, toggle, address and command
            raw: (0b11 << 12) | (1 << 11) | (20 << 6) | 9,
        }
    );
}

#[test]
fn protocol_id_from_u8() {
    use infrared::protocol::UnknownProtocol;

    for id in [ProtocolId::Nec, ProtocolId::Rc5, ProtocolId::Denon] {
        assert_eq!(ProtocolId::try_from(id as u8), Ok(id));
    }
    assert_eq!(ProtocolId::try_from(0), Err(UnknownProtocol(0)));
    assert_eq!(ProtocolId::Rc5.to_string(), "RC5");
}
