use crate::{
    recv::{InfraredReceiver, State}
};

pub struct BufferReceiver<'a> {
    buf: &'a [u16],
    scale_factor: u32,
}

impl<'a> BufferReceiver<'a> {
    /// Create a new BufferReceiver with `buf` as the underlying value change buffer
    pub fn new(buf: &'a [u16], samplerate: u32) -> Self {
        Self {
            buf,
            scale_factor: crate::TIMEBASE / samplerate,
        }
    }

    /// Create an iterator over the buffer with `Prococol` as decoder
    pub fn iter<Protocol: InfraredReceiver>(&self) -> BufferIterator<'a, Protocol> {
        BufferIterator {
            buf: &self.buf,
            scaler: self.scale_factor,
            pos: 0,
            sm: Protocol::create(),
        }
    }
}

pub struct BufferIterator<'a, Protocol> {
    buf: &'a [u16],
    pos: usize,
    scaler: u32,
    sm: Protocol,
}

impl<'a, Protocol: InfraredReceiver> Iterator for BufferIterator<'a, Protocol> {
    type Item = Protocol::Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = u32::from(self.buf[self.pos]) * self.scaler;
            self.pos += 1;

            let state: State = self.sm.event(pos_edge, dt_us).into();

            match state {
                State::Idle | State::Receiving => {
                    continue;
                }
                State::Done => {
                    let cmd = self.sm.command();
                    self.sm.reset();
                    break cmd;
                }
                State::Error(_) => {
                    self.sm.reset();
                    break None;
                }
            }
        }
    }
}
