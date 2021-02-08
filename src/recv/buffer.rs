use crate::{
    recv::{InfraredReceiver, Status}
};
use crate::recv::InfraredReceiverState;

pub struct BufferReceiver<'a> {
    buf: &'a [u16],
    samplerate: u32,
}

impl<'a> BufferReceiver<'a> {
    /// Create a new BufferReceiver with `buf` as the underlying value change buffer
    pub fn new(buf: &'a [u16], samplerate: u32) -> Self {
        Self {
            buf,
            samplerate,
        }
    }

    /// Create an iterator over the buffer with `Prococol` as decoder
    pub fn iter<Protocol: InfraredReceiver>(&self) -> BufferIterator<'a, Protocol> {
        BufferIterator {
            buf: &self.buf,
            pos: 0,
            receiver_state: Protocol::receiver_state(self.samplerate),
        }
    }
}

pub struct BufferIterator<'a, Protocol>
where Protocol: InfraredReceiver,
{
    buf: &'a [u16],
    pos: usize,
    receiver_state: Protocol::ReceiverState,
}

impl<'a, Protocol: InfraredReceiver> Iterator for BufferIterator<'a, Protocol> {
    type Item = Protocol::Cmd;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.pos == self.buf.len() {
                break None;
            }

            let pos_edge = self.pos & 0x1 == 0;
            let dt_us = u32::from(self.buf[self.pos]);
            self.pos += 1;

            let state: Status = Protocol::event(
                &mut self.receiver_state,
                pos_edge, dt_us).into();

            match state {
                Status::Idle | Status::Receiving => {
                    continue;
                }
                Status::Done => {
                    let cmd = Protocol::command(
                        &self.receiver_state
                    );
                    self.receiver_state.reset(
                    );
                    break cmd;
                }
                Status::Error(_) => {
                    self.receiver_state.reset(
                    );
                    break None;
                }
            }
        }
    }
}
