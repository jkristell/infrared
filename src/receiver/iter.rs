use core::marker::PhantomData;

use crate::{
    receiver::{time::InfraMonotonic, DecoderFactory, ProtocolDecoder, State},
    Protocol,
};

pub struct BufferIterator<'a, Proto, Mono, Cmd>
where
    Proto: DecoderFactory<Mono>,
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
    Proto: DecoderFactory<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    pub fn new(freq: u32, buf: &'a [Mono::Duration]) -> Self {
        BufferIterator {
            pos: 0,
            buf,
            decoder: Proto::decoder(freq),
            cmd: PhantomData,
        }
    }
}

impl<'a, Proto, Mono, Cmd> Iterator for BufferIterator<'a, Proto, Mono, Cmd>
where
    Proto: DecoderFactory<Mono>,
    Mono: InfraMonotonic,
    Cmd: From<<Proto as Protocol>::Cmd>,
{
    type Item = Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = self.buf[self.pos];
            self.pos += 1;

            let state = self.decoder.event(pos_edge, dt_us);

            match state {
                State::Idle | State::Receiving => {
                    continue;
                }
                State::Done => {
                    let cmd = self.decoder.command();
                    self.decoder.reset();
                    break cmd.map(|r| r.into());
                }
                State::Error(_) => {
                    self.decoder.reset();
                    break None;
                }
            }
        }
    }
}
