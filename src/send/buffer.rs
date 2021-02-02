use core::convert::TryFrom;
use crate::send::ToPulsedata;

pub struct PulsedataBuffer {
    pub buf: [u16; 96],
    pub offset: usize,
    pub scaler: u16,
}

impl PulsedataBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; 96],
            offset: 0,
            scaler: 1,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn with_samplerate(samplerate: u32) -> Self {
        Self {
            buf: [0; 96],
            offset: 0,
            scaler: u16::try_from(1000 / (samplerate / 1000)).unwrap(),
        }
    }

    pub fn load(&mut self, c: &impl ToPulsedata) {
        let len = c.to_pulsedata(&mut self.buf[self.offset..]);

        // Apply the scaling on the buf
        for elem in &mut self.buf[self.offset .. self.offset + len] {
            *elem /= self.scaler;
        }

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
            index: 0,
        }
    }
}

pub struct PulseIterator<'a> {
    pulses: &'a [u16],
    index: usize,
}

impl<'a> Iterator for PulseIterator<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.pulses.len() {
            None
        } else {
            let r = self.pulses[self.index];
            self.index += 1;
            Some(r)
        }
    }
}
