use crate::{recv::State, PulseLengths, ReceiverSM};
use core::marker::PhantomData;

const BUFRECVBUFLEN: usize = 512;

pub struct BufferReceiver<PROTOCOL> {
    buf: [u16; BUFRECVBUFLEN],
    len: usize,
    scaler: u32,
    _sm: PhantomData<PROTOCOL>,
}

impl<PROTOCOL: ReceiverSM> BufferReceiver<PROTOCOL> {
    /// Create a new BufferReceiver with initial value change buffer
    pub fn with_values(values: &[u16], samplerate: u32) -> Self {
        let mut buf = [0; 512];

        let len = core::cmp::min(BUFRECVBUFLEN, values.len());
        buf[0..len].copy_from_slice(values);

        Self {
            buf,
            len,
            scaler: crate::TIMEBASE / samplerate,
            _sm: PhantomData,
        }
    }

    /// Add command to buffer
    /// Panics if not enough room in Buffer
    pub fn add_cmd(&mut self, cmd: &impl PulseLengths) {
        let cmdlen = cmd.encode(&mut self.buf[self.len..]);
        self.len += cmdlen
    }

    /// Add values
    /// Panics if not enough room in Buffer
    pub fn add(&mut self, values: &[u16]) {
        self.buf[self.len..values.len()].copy_from_slice(values);
        self.len += values.len();
    }

    pub fn iter(&self) -> BufferIterator<'_, PROTOCOL> {
        self.into_iter()
    }
}

impl<'a, PROTOCOL: ReceiverSM> IntoIterator for &'a BufferReceiver<PROTOCOL> {
    type Item = PROTOCOL::Cmd;
    type IntoIter = BufferIterator<'a, PROTOCOL>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator {
            buf: &self.buf,
            scaler: self.scaler,
            pos: 0,
            sm: PROTOCOL::create(),
        }
    }
}

pub struct BufferIterator<'a, PROTOCOL> {
    buf: &'a [u16],
    pos: usize,
    scaler: u32,
    sm: PROTOCOL,
}

impl<'a, PROTOCOL: ReceiverSM> Iterator for BufferIterator<'a, PROTOCOL> {
    type Item = PROTOCOL::Cmd;

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
