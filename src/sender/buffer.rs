use crate::sender::ProtocolEncoder;

pub(crate) struct PulseBuffer<const S: usize> {
    pub buf: [u32; S],
    pub offset: usize,
}

impl<const S: usize> PulseBuffer<S> {
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

    /// Get the pulse length at `index`, if it is part of the loaded pulse train
    pub fn get(&self, index: usize) -> Option<u32> {
        self.buffer().get(index).copied()
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buf[..self.offset]
    }
}

impl<'a, const S: usize> IntoIterator for &'a PulseBuffer<S> {
    type Item = u32;
    type IntoIter = core::iter::Copied<core::slice::Iter<'a, u32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer().iter().copied()
    }
}
