use crate::send::{InfraredSender};

pub(crate) struct PulsedataBuffer {
    pub buf: [u16; 96],
    pub offset: usize,
}

impl PulsedataBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; 96],
            offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn load<SendProto: InfraredSender>(&mut self, state: &SendProto::State, c: &SendProto::Cmd) {
        let len = SendProto::cmd_pulsedata(state, c, &mut self.buf[self.offset..]);
        self.offset += len;
    }

    pub fn get(&self, index: usize) -> Option<u16> {
        self.buf.get(index).cloned()
    }

    pub fn buffer(&self) -> &[u16] {
        &self.buf[..self.offset]
    }
}

impl<'a> IntoIterator for &'a PulsedataBuffer {
    type Item = u16;
    type IntoIter = PulseIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PulseIterator {
            pulses: &self.buf[0..self.offset],
            pos: 0,
        }
    }
}

pub struct PulseIterator<'a> {
    pulses: &'a [u16],
    pos: usize,
}

impl<'a> Iterator for PulseIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.pulses.len() {
            None
        } else {
            let r = self.pulses[self.pos];
            self.pos += 1;
            Some(r)
        }
    }
}
