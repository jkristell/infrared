use crate::sender::ProtocolEncoder;

pub(crate) struct PulsedataBuffer<const S: usize> {
    pub buf: [u32; S],
    pub offset: usize,
}

impl<const S: usize> PulsedataBuffer<S> {
    pub fn new() -> Self {
        Self {
            buf: [0; S],
            offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn load<SendProto: ProtocolEncoder<F>, const F: u32>(&mut self, c: &SendProto::Cmd) {
        let len = SendProto::encode(c, &mut self.buf[self.offset..]);
        self.offset += len;
    }

    pub fn get(&self, index: usize) -> Option<u32> {
        self.buf.get(index).cloned()
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buf[..self.offset]
    }
}

impl<'a, const S: usize> IntoIterator for &'a PulsedataBuffer<S> {
    type Item = u32;
    type IntoIter = PulseIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PulseIterator {
            pulses: &self.buf[0..self.offset],
            pos: 0,
        }
    }
}

pub struct PulseIterator<'a> {
    pulses: &'a [u32],
    pos: usize,
}

impl Iterator for PulseIterator<'_> {
    type Item = u32;

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
