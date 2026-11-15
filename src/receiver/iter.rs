use core::{iter::FusedIterator, marker::PhantomData};

use crate::{
    Level,
    receiver::{time::TimeBase, DecoderBuilder, ProtocolDecoder, DecoderState},
    Protocol,
};

/// Iterator over the commands decoded from a buffer of pulse durations
///
/// Frames that fail to decode are skipped.
///
/// Returned by [`BufferReceiver::iter`](crate::receiver::BufferReceiver::iter).
pub struct BufferIterator<'a, Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pos: usize,
    buf: &'a [Time::Duration],
    pub(crate) decoder: Proto::Decoder,
    cmd: PhantomData<Cmd>,
}

impl<'a, Proto, Time, Cmd> BufferIterator<'a, Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(resolution: u32, buf: &'a [Time::Duration]) -> Self {
        BufferIterator {
            pos: 0,
            buf,
            decoder: Proto::build(resolution),
            cmd: PhantomData,
        }
    }
}

impl<Proto, Time, Cmd> Iterator for BufferIterator<'_, Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
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

impl<Proto, Time, Cmd> FusedIterator for BufferIterator<'_, Proto, Time, Cmd>
where
    Proto: DecoderBuilder<Time>,
    Time: TimeBase,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
}
