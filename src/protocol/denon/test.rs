use crate::{
    protocol::{Denon, denon::DenonCommand},
    receiver::{BufferReceiver, DecodingError},
};

#[test]
fn denon() {
    use std::vec::Vec;

    let dists = &[
        0, 136, 65, 18, 16, 18, 15, 19, 49, 18, 15, 19, 48, 19, 15, 19, 48, 19, 15, 18, 15, 19, 48,
        19, 15, 19, 14, 19, 49, 18, 49, 18, 15, 19, 15, 19, 14, 19, 15, 19, 15, 18, 15, 19, 15, 19,
        14, 19, 48, 19, 15, 19, 49, 18, 16, 17, 16, 18, 16, 18, 16, 17, 16, 18, 16, 18, 15, 18, 49,
        18, 16, 17, 17, 17, 16, 18, 16, 17, 16, 18, 16, 18, 15, 18, 16, 17, 17, 17, 16, 18, 16, 18,
        15, 18, 16, 18, 49, 18, 16, 18, 2939, 138, 65, 17, 16, 18, 16, 18, 49, 18, 16, 17, 50, 18,
        15, 18, 50, 17, 16, 18, 16, 17, 50, 17, 16, 18, 16, 17, 18, 49, 18, 16, 16, 17, 18, 16, 18,
        16, 17, 16, 19, 15, 18, 15, 18, 16, 17, 50, 19, 15, 19, 48, 17, 16, 19, 15, 18, 15, 19, 15,
        18, 16, 18, 15, 19, 15, 19, 48, 19, 14, 19, 15, 19, 15, 19, 14, 19, 15, 19, 14, 19, 15, 19,
        14, 19, 15, 19, 15, 18, 15, 19, 15, 19, 14, 20, 47, 19, 15, 19, 2940, 136, 65, 19, 15, 18,
        15, 19, 49, 18, 15, 19, 48, 19, 15, 19, 48, 19, 15, 18, 15, 19, 48, 19, 15, 18, 15, 19, 49,
        18, 49, 18, 15, 19, 15, 19, 14, 19, 15, 19, 15, 18, 15, 19, 15, 19, 14, 19, 48, 19, 15, 19,
        48, 19, 15, 18, 17, 16, 19, 15, 19, 15, 18, 15, 19, 49, 19, 15, 18, 16, 17, 16, 18, 16, 18,
        15, 18, 16, 19, 14, 18, 16, 18, 16, 17, 16, 18, 16, 18, 15, 18, 16, 18, 49, 18, 16, 18,
    ];
    let brecv = BufferReceiver::<Denon>::with_resolution(40_000);
    let cmds = brecv.iter(dists).collect::<Vec<_>>();

    // The second frame has a corrupted vendor id and is rejected, the third
    // is cut short in the capture
    assert_eq!(cmds, [DenonCommand::new(0x014, 0x01)]);
    assert_eq!(cmds[0].pack(), 0x4001_0140_3254);
}

#[test]
fn unpack_validates() {
    let bits = DenonCommand::new(0xABC, 0x5D).pack();
    assert_eq!(
        DenonCommand::unpack(bits),
        Ok(DenonCommand::new(0xABC, 0x5D))
    );
    // Wrong vendor
    assert_eq!(DenonCommand::unpack(bits ^ 1), Err(DecodingError::Address));
    // Wrong parity
    assert_eq!(
        DenonCommand::unpack(bits ^ (1 << 47)),
        Err(DecodingError::Validation)
    );
}
