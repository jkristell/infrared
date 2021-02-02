use crate::{
    recv::{InfraredReceiver, State}
};

pub struct BufferReceiver<'a> {
    buf: &'a [u16],
    scaler: u32,
}

impl<'a> BufferReceiver<'a> {
    /// Create a new BufferReceiver with initial value change buffer
    pub fn with_values(buf: &'a [u16], samplerate: u32) -> Self {

        Self {
            buf,
            scaler: crate::TIMEBASE / samplerate,
        }
    }

    pub fn iter<Protocol: InfraredReceiver>(&self) -> BufferIterator<'a, Protocol> {
        BufferIterator {
            buf: &self.buf,
            scaler: self.scaler,
            pos: 0,
            sm: Protocol::create(),
        }
    }


    // Add command to buffer
    // Panics if not enough room in Buffer
    //pub fn add_cmd(&mut self, cmd: &impl ToPulsedata) {
    //    let cmdlen = cmd.to_pulsedata(&mut self.buf[self.len..]);
    //    self.len += cmdlen
    //}

    // Add values
    // Panics if not enough room in Buffer
    //pub fn add(&mut self, values: &[u16]) {
    //    self.buf[self.len..values.len()].copy_from_slice(values);
    //    self.len += values.len();
    //}

    //pub fn iter(&self) -> BufferIterator<'_, Protocol> {
    //    self.into_iter()
    //}
}

/*
impl<'a, Protocol: ReceiverSM> IntoIterator for &'a BufferReceiver<'a, Protocol> {
    type Item = Protocol::Cmd;
    type IntoIter = BufferIterator<'a, Protocol>;

    fn into_iter(self) -> Self::IntoIter {
        BufferIterator {
            buf: &self.buf,
            scaler: self.scaler,
            pos: 0,
            sm: Protocol::create(),
        }
    }
}
*/
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
