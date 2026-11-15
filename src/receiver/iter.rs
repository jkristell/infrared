use core::{iter::FusedIterator, marker::PhantomData};

use crate::{
    Level,
    receiver::{time::InfraMonotonic, DecoderBuilder, ProtocolDecoder, DecoderState},
    Protocol,
};

/// Iterator over the commands decoded from a buffer of pulse durations
///
/// Frames that fail to decode are skipped.
///
/// Returned by [`BufferReceiver::iter`](crate::receiver::BufferReceiver::iter).
pub struct BufferIterator<'a, Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pos: usize,
    buf: &'a [Mono::Duration],
    pub(crate) decoder: Proto::Decoder,
    cmd: PhantomData<Cmd>,
}

impl<'a, Proto, Mono, Cmd> BufferIterator<'a, Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(resolution: u32, buf: &'a [Mono::Duration]) -> Self {
        BufferIterator {
            pos: 0,
            buf,
            decoder: Proto::build(resolution),
            cmd: PhantomData,
        }
    }
}

impl<Proto, Mono, Cmd> Iterator for BufferIterator<'_, Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    type Item = Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            // The buffer starts with a mark
            let level = if self.pos & 0x1 == 0 { Level::Mark } else { Level::Space };
            let dt = self.buf[self.pos];
            self.pos += 1;

            let state = self.decoder.event(dt, level);

            match state {
                DecoderState::Idle | DecoderState::Receiving => {
                    continue;
                }
                DecoderState::Done => {
                    let cmd = self.decoder.command();
                    self.decoder.reset();
                    break cmd.map(|r| r.into());
                }
                DecoderState::Error(_) => {
                    self.decoder.reset();
                    continue;
                }
            }
        }
    }
}

impl<Proto, Mono, Cmd> FusedIterator for BufferIterator<'_, Proto, Mono, Cmd>
where
    Proto: DecoderBuilder<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
}
