use crate::{protocols::nec::{Nec, Nec16, NecCommand, NecSamsung, NecStandard, NecVariant}, bufrecv::BufferReceiver, Command};
use crate::sender::PulseBuffer;

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

#[test]
fn all_commands() {
    let mut ptb = PulseBuffer::with_samplerate(40_000);

    for address in 0..255 {
        for cmdnum in 0..255 {
            let cmd: NecCommand<NecStandard> = NecCommand::new(address, cmdnum);
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

        let cmd: NecCommand<NecStandard> = NecCommand::new(20, 10);
        ptb.load(&cmd);

        let receiver: BufferReceiver<Nec> = BufferReceiver::with_values(&ptb.buf, *samplerate);

        if let Some(cmd) = receiver.iter().next() {
            println!("{:?}", cmd);
            assert_eq!(cmd.address(), 20);
            assert_eq!(cmd.data(), 10);
        } else {
            panic!("Failed to parse command at samplerate: {}", samplerate)
        }
    }
}

