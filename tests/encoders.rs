use infrared::{
    protocol::Protocol,
    receiver::{BufferReceiver, DecoderBuilder},
    sender::{ProtocolEncoder, PulseSender},
};

/// Encode `cmd` at `resolution` Hz and decode it again
fn roundtrip<P>(cmd: &P::Cmd, resolution: u32) -> Vec<P::Cmd>
where
    P: ProtocolEncoder + DecoderBuilder<u32>,
{
    let mut sender: PulseSender = PulseSender::new(resolution);
    sender.load::<P>(cmd);
    BufferReceiver::<P>::with_resolution(resolution)
        .iter(sender.buffer())
        .collect()
}

/// The loaded pulse train in µs, without the leading zero
fn pulses<P: ProtocolEncoder>(cmd: &<P as Protocol>::Cmd) -> Vec<u32> {
    let mut sender: PulseSender = PulseSender::new(1_000_000);
    sender.load::<P>(cmd);
    sender.buffer()[1..].to_vec()
}

#[cfg(feature = "nec")]
#[test]
fn nec_frame_ends_with_stop_mark() {
    use infrared::protocol::{Nec, nec::NecCommand};

    let p = pulses::<Nec>(&NecCommand {
        addr: 1,
        cmd: 2,
        repeat: false,
    });
    // Header, 32 bits and the stop mark
    assert_eq!(p.len(), 67);
    assert_eq!(p[..2], [9000, 4500]);
    assert_eq!(*p.last().unwrap(), 560);
}

#[cfg(feature = "nec")]
#[test]
fn nec_repeat() {
    use infrared::protocol::{Nec, nec::NecCommand};

    let repeat = NecCommand {
        addr: 1,
        cmd: 2,
        repeat: true,
    };
    assert_eq!(pulses::<Nec>(&repeat), [9000, 2250, 560]);

    // A frame followed by a repeat 40 ms later
    let cmd = NecCommand {
        addr: 1,
        cmd: 2,
        repeat: false,
    };
    let mut train = vec![0];
    train.extend(pulses::<Nec>(&cmd));
    train.push(40_000);
    train.extend(pulses::<Nec>(&repeat));

    let cmds: Vec<_> = BufferReceiver::<Nec>::with_resolution(1_000_000)
        .iter(&train)
        .collect();
    assert_eq!(cmds, [cmd, repeat]);
}

#[cfg(feature = "nec")]
#[test]
fn samsung_nec_repeats_the_full_frame() {
    use infrared::protocol::{nec::SamsungNecCommand, SamsungNec};

    let cmd = SamsungNecCommand { addr: 7, cmd: 2, repeat: false };
    let repeat = SamsungNecCommand { repeat: true, ..cmd };
    assert_eq!(pulses::<SamsungNec>(&repeat), pulses::<SamsungNec>(&cmd));
}

#[cfg(feature = "rc5")]
#[test]
fn rc5_frame_ends_with_full_mark() {
    use infrared::protocol::{Rc5, rc5::Rc5Command};

    // Last bit is one: a space followed by a mark
    let p = pulses::<Rc5>(&Rc5Command::new(0, 1, false));
    assert_eq!(p.len() % 2, 1);
    assert_eq!(*p.last().unwrap(), 889);
}

#[cfg(feature = "rc6")]
#[test]
fn rc6_high_resolution() {
    use infrared::protocol::{Rc6, rc6::Rc6Command};

    let cmd = Rc6Command::new(70, 20);
    for res in [40_000, 1_000_000, 48_000_000] {
        let cmds = roundtrip::<Rc6>(&cmd, res);
        assert_eq!(cmds.len(), 1, "at {res} Hz");
        assert_eq!(cmds[0], cmd, "at {res} Hz");
    }
}

#[cfg(feature = "denon")]
#[test]
fn denon_roundtrip() {
    use infrared::protocol::{Denon, denon::DenonCommand};

    for bits in [0, 1, 0x2A4C_0280_E86E, 0xFFFF_FFFF_FFFF] {
        for res in [20_000, 1_000_000] {
            let cmds = roundtrip::<Denon>(&DenonCommand { bits }, res);
            assert_eq!(cmds.len(), 1);
            assert_eq!(cmds[0].bits, bits, "at {res} Hz");
        }
    }
}

#[cfg(feature = "sbp")]
#[test]
fn sbp_roundtrip() {
    use infrared::protocol::{Sbp, sbp::SbpCommand};

    for (address, command) in [(0, 0), (0x2CA3, 0x2E), (0xFFFF, 0xFF)] {
        for res in [20_000, 1_000_000] {
            let cmd = SbpCommand {
                address,
                command,
                valid: true,
            };
            let cmds = roundtrip::<Sbp>(&cmd, res);
            assert_eq!(cmds.len(), 1);
            assert_eq!(
                (cmds[0].address, cmds[0].command, cmds[0].valid),
                (address, command, true),
                "at {res} Hz"
            );
        }
    }
}
