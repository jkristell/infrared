use crate::protocols::nec::cmds::{Nec16Command, NecAppleCommand, NecCommand, NecSamsungCommand};
use crate::protocols::nec::NecCommandTrait;
use crate::sender::PulseBuffer;
use crate::{bufrecv::BufferReceiver, protocols::nec::Nec};

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

    let brecv: BufferReceiver<Nec> = BufferReceiver::with_values(&dists, 40_000);
    let cmds = brecv.iter().collect::<Vec<_>>();
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
    let bits: u32 = cmd.pack();

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

    let bits: u32 = cmd.pack();
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
    let bits: u32 = cmd.pack();

    assert!(Nec16Command::validate(bits));

    assert_eq!(bits, 0x23DC6DD2);
    assert_eq!((bits >> 24) & 0xFF, (!(bits >> 16) & 0xFF));

    let cmd2 = Nec16Command::unpack(bits, false).unwrap();
    assert_eq!(cmd, cmd2);
}

#[test]
fn all_nec_commands() {
    let mut ptb = PulseBuffer::with_samplerate(40_000);

    for address in 0..255 {
        for cmdnum in 0..255 {
            let cmd = NecCommand {
                addr: address,
                cmd: cmdnum,
                repeat: false,
            };
            ptb.load(&cmd);
            let brecv: BufferReceiver<Nec> = BufferReceiver::with_values(&ptb.buf, 40_000);

            let cmdres = brecv.iter().next().unwrap();
            assert_eq!(cmd.addr, cmdres.addr);
            assert_eq!(cmd.cmd, cmdres.cmd);
        }
    }
}

#[test]
fn test_samplerates() {
    let samplerates = [20_000, 40_000, 80_000];

    for samplerate in &samplerates {
        let mut ptb = PulseBuffer::with_samplerate(*samplerate);

        let cmd: NecCommand = NecCommand {
            addr: 20,
            cmd: 10,
            repeat: false,
        };
        ptb.load(&cmd);

        let receiver: BufferReceiver<Nec> = BufferReceiver::with_values(&ptb.buf, *samplerate);

        if let Some(cmd) = receiver.iter().next() {
            println!("{:?}", cmd);
            assert_eq!(cmd.addr, 20);
            assert_eq!(cmd.cmd, 10);
        } else {
            panic!("Failed to parse command at samplerate: {}", samplerate)
        }
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
        assert!(NecAppleCommand::validate(*bits));

        let cmd = NecAppleCommand::unpack(*bits, false).unwrap();

        assert_eq!(cmd.command_page, 0xE);
        assert_eq!(cmd.command, *cmdnum);
    }
}
