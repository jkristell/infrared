#![cfg(all(feature = "rc5", feature = "remotes"))]

use infrared::{
    protocol::rc5::Rc5Command,
    remotecontrol::{rc5::CdPlayer, Action, Button, RemoteControl},
};

#[test]
fn remote_encode_decode_roundtrip() {
    // Encode a button press into a command
    let cmd = CdPlayer::encode(&Action::Next).unwrap();
    assert_eq!(cmd.address, 20);
    assert_eq!(cmd.command, 32);

    // And map the command back to a button
    let button: Button<CdPlayer> = cmd.into();
    assert_eq!(button.action(), Some(Action::Next));
    assert!(!button.is_repeat());
}

#[test]
fn remote_ignores_other_address() {
    // Same command id, wrong address
    let cmd = Rc5Command::new(10, 32, false);
    assert_eq!(CdPlayer::decode(&cmd), None);

    let button: Button<CdPlayer> = cmd.into();
    assert_eq!(button.action(), None);
}

#[test]
fn remote_unmapped_command() {
    // Right address, command id without a button mapping
    let cmd = Rc5Command::new(20, 63, false);
    assert_eq!(CdPlayer::decode(&cmd), None);
}
