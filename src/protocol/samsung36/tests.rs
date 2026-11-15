use std::vec::Vec;

use crate::{
    protocol::Samsung36,
    receiver::{BufferReceiver, DecodingError, Receiver},
};

/// Build a Samsung36 pulse train as edge distances in µs, starting with a rising edge
fn pulsetrain(address: u16, cmd: u8, checksum_ok: bool, dists: &mut Vec<u32>) {
    // Leading rising edge
    dists.push(0);
    // Sync: 4500 high + 4500 low
    dists.push(4500);
    dists.push(4500);
    // 16 address bits, LSB first. Bit: 500 high + 500/1500 low
    for i in 0..16 {
        dists.push(500);
        dists.push(if address & (1 << i) != 0 { 1500 } else { 500 });
    }
    // Divider: 500 high + 4500 low
    dists.push(500);
    dists.push(4500);
    // 20 command bits, LSB first: 4 unknown, cmd, then the checksum
    // (inverted cmd when valid)
    let check = if checksum_ok { !cmd } else { cmd };
    let command: u32 = (u32::from(cmd) << 4) | (u32::from(check) << 12);
    for i in 0..20 {
        dists.push(500);
        dists.push(if command & (1 << i) != 0 { 1500 } else { 500 });
    }
}

#[test]
fn decode() {
    let mut dists = Vec::new();
    pulsetrain(0xA55A, 0x37, true, &mut dists);

    let r = BufferReceiver::<Samsung36>::with_resolution(1_000_000);
    let cmds: Vec<_> = r.iter(&dists).collect();

    assert_eq!(cmds.len(), 1);
    assert_eq!(cmds[0].address, 0xA55A);
    assert_eq!(cmds[0].command, 0x37);
}

#[test]
fn decode_all_commands() {
    for cmd in 0..=255 {
        let mut dists = Vec::new();
        pulsetrain(0x0101, cmd, true, &mut dists);

        let r = BufferReceiver::<Samsung36>::with_resolution(1_000_000);
        let cmds: Vec<_> = r.iter(&dists).collect();

        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].address, 0x0101);
        assert_eq!(cmds[0].command, cmd);
    }
}

#[test]
fn invalid_checksum() {
    let mut dists = Vec::new();
    pulsetrain(0xA55A, 0x37, false, &mut dists);

    let mut r: Receiver<Samsung36> = Receiver::new(1_000_000);
    let results: Vec<_> = dists
        .iter()
        .enumerate()
        .map(|(i, dt)| r.event(*dt, i % 2 == 0))
        .filter(|res| !matches!(res, Ok(None)))
        .collect();

    assert_eq!(results, [Err(DecodingError::Validation)]);
}
