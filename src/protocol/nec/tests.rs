use fugit::{TimerDurationU32, TimerInstantU32};

use crate::{
    protocol::{
        nec::{AppleNecCommand, Nec16Command, NecCommand, NecCommandVariant, NecSamsungCommand},
        Nec,
    },
    receiver::BufferInputReceiver,
    sender::PulsedataBuffer,
    Receiver,
};

#[test]
#[rustfmt::skip]
fn standard_nec() {
    use std::vec::Vec;

    let dists = [
        0, 363, 177,
        24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21, 24, 66, 24,
        66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 24, 21,
        24, 21, 24, 21, 24, 21, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66, 24,

        0, 363, 177,
        24, 21, 24, 21, 24, 21, 24, 21, 24, 21, 24, 20, 24, 21, 24, 21, 24, 66, 24,
        66, 24, 65, 25, 65, 24, 66, 24, 66, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 24, 21,
        24, 21, 24, 21, 24, 21, 24, 65, 25, 65, 24, 21, 24, 21, 24, 66, 24, 65, 25, 65, 24, 66, 24,
    ];

    let mut brecv = BufferInputReceiver::<Nec>::with_frequenzy(40_000);

    let cmds = brecv.iter(&dists).collect::<Vec<_>>();
    assert_eq!(cmds.len(), 2);

    for cmd in &cmds {
        assert_eq!(cmd.addr, 0);
        assert_eq!(cmd.cmd, 12);
    }
}

#[test]
fn cmd_standard() {
    let cmd = NecCommand {
        addr: 7,
        cmd: 44,
        repeat: false,
    };
    let bits = cmd.pack();

    assert!(NecCommand::validate(bits));

    assert_eq!(bits, 0xD32CF807);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (!bits & 0xFF));

    let cmd2 = NecCommand::unpack(bits, false).unwrap();

    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_samsumg() {
    let cmd = NecSamsungCommand {
        addr: 7,
        cmd: 44,
        repeat: false,
    };

    let bits = cmd.pack();
    NecSamsungCommand::validate(bits);

    assert_eq!(bits, 0xD32C0707);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));
    assert_eq!((bits >> 8) & 0xFF, (bits & 0xFF));

    let cmd2 = NecSamsungCommand::unpack(bits, false).unwrap();
    assert_eq!(cmd, cmd2);
}

#[test]
fn cmd_nec16() {
    let cmd = Nec16Command {
        addr: 28114,
        cmd: 220,
        repeat: false,
    };
    let bits = cmd.pack();

    assert!(Nec16Command::validate(bits));

    assert_eq!(bits, 0x23DC6DD2);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));

    let cmd2 = Nec16Command::unpack(bits, false).unwrap();
    assert_eq!(cmd, cmd2);
}

#[test]
fn all_nec_commands() {
    const FREQUENCY: u32 = 40_000;
    let mut ptb = PulsedataBuffer::<96>::new();

    for address in 0..255 {
        for cmdnum in 0..255 {
            ptb.reset();
            let cmd = NecCommand {
                addr: address,
                cmd: cmdnum,
                repeat: false,
            };
            ptb.load::<Nec, FREQUENCY>(&cmd);
            let mut brecv = BufferInputReceiver::<Nec>::with_frequenzy(40_000);

            let cmdres = brecv.iter(ptb.buffer()).next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

#[test]
fn clock_frequencies() {
    one_freq::<20_000>();
    one_freq::<40_000>();
    one_freq::<48_000_000>();
}

fn one_freq<const F: u32>() {
    let mut ptb = PulsedataBuffer::<96>::new();

    let cmd: NecCommand = NecCommand {
        addr: 20,
        cmd: 10,
        repeat: false,
    };
    ptb.load::<Nec, F>(&cmd);

    let mut receiver = BufferInputReceiver::<Nec>::with_frequenzy(F);

    if let Some(cmd) = receiver.iter(ptb.buffer()).next() {
        assert_eq!(cmd.addr, 20);
        assert_eq!(cmd.cmd, 10);
    } else {
        panic!("Failed to parse command at samplerate: {}.", F)
    }
}

#[test]
fn cmd_apple2009() {
    let tests: &[(u32, u8)] = &[
        (0x9B0987EE, 0x04), // Left
        (0x9B0A87EE, 0x05), // Up
        (0x9B0687EE, 0x03), // Right
        (0x9B0C87EE, 0x06), // Down
        (0x9B5C87EE, 0x2e), // 0x2e select button prefix
        (0x9B0587EE, 0x02), // play pause?
        (0x9B0387EE, 0x01), // Menu
        (0x9B5F87EE, 0x2F), // Play/Pause Prefix
        (0x9B0587EE, 0x02), // Play/Pause
    ];

    for (bits, cmdnum) in tests {
        assert!(AppleNecCommand::validate(*bits));

        let cmd = AppleNecCommand::unpack(*bits, false).unwrap();

        assert_eq!(cmd.command_page, 0xE);
        assert_eq!(cmd.command, *cmdnum);
    }
}

#[test]
fn repeat() {
    #[rustfmt::skip]
    let data = [
        // Command
        0, 9130, 4532, 571, 562, 571, 562, 571, 562, 570, 563, 572, 562, 571, 562, 570, 562, 571,
        562, 571, 1699, 570, 1697, 571, 1697, 571, 1698, 572, 1697, 570, 1699, 570, 1698, 571,
        1698, 571, 562, 571, 563, 569, 564, 571, 1697, 571, 1698, 571, 562, 572, 562, 571, 562,
        571, 1697, 571, 1697, 571, 1698, 570, 563, 571, 562, 569, 1698, 571, 1698, 570, 1699, 571,
        // Repeats
        40648, 9124, 2260, 571,
        97387, 9123, 2259, 571,
        97385, 9125, 2260, 571,
        97398, 9126, 2260, 571,
        97380, 9120, 2258, 571,
        97373, 9124, 2259, 572,
        97409, 9124, 2258, 571,
        97387
    ];

    let mut receiver = BufferInputReceiver::<Nec>::with_frequenzy(1_000_000);

    let iter = receiver.iter(&data);

    let cmds = iter.collect::<std::vec::Vec<_>>();

    assert_eq!(cmds.len(), 8);
    assert_eq!(cmds[0].repeat, false);
    assert_eq!(cmds[1].repeat, true);
    assert_eq!(cmds[7].repeat, true);
}

#[test]
fn fugit() {
    #[rustfmt::skip]
        let data = [
        // Command
        0, 9130, 4532, 571, 562, 571, 562, 571, 562, 570, 563, 572, 562, 571, 562, 570, 562, 571,
        562, 571, 1699, 570, 1697, 571, 1697, 571, 1698, 572, 1697, 570, 1699, 570, 1698, 571,
        1698, 571, 562, 571, 563, 569, 564, 571, 1697, 571, 1698, 571, 562, 572, 562, 571, 562,
        571, 1697, 571, 1697, 571, 1698, 570, 563, 571, 562, 569, 1698, 571, 1698, 570, 1699, 571,
        // Repeats
        40648, 9124, 2260, 571,
        97387, 9123, 2259, 571,
        97385, 9125, 2260, 571,
        97398, 9126, 2260, 571,
        97380, 9120, 2258, 571,
        97373, 9124, 2259, 572,
        97409, 9124, 2258, 571,
        97387
    ];

    let mut receiver = Receiver::builder()
        .nec()
        .monotonic::<TimerInstantU32<1_000_000>>()
        .build();

    let mut cmds = std::vec::Vec::new();

    let mut edge = false;
    for dt in &data {
        edge = !edge;

        let dtf = TimerDurationU32::from_ticks(*dt);

        let s = receiver.event_edge(dtf, edge);

        if let Ok(Some(cmd)) = s {
            cmds.push(cmd);
        }
    }

    assert_eq!(cmds.len(), 8);
    assert_eq!(cmds[0].repeat, false);
    assert_eq!(cmds[1].repeat, true);
    assert_eq!(cmds[7].repeat, true);
}
