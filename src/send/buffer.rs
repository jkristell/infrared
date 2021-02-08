use core::convert::TryFrom;
use crate::send::{ToPulsedata, InfraredSender};

pub struct PulsedataBuffer<SendProto: InfraredSender> {
    pub buf: [u16; 96],
    pub offset: usize,
    pub proto: SendProto,
}

impl<SendProto: InfraredSender> PulsedataBuffer<SendProto> {
    pub fn new(samplerate: u32) -> Self {
        Self {
            buf: [0; 96],
            offset: 0,
            proto: SendProto::with_samplerate(samplerate)
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0;
    }

    pub fn with_samplerate(samplerate: u32) -> Self {
        Self::new(samplerate)
    }

    pub fn load(&mut self, c: &SendProto::Cmd) {
        let len = self.proto.cmd_pulsedata(c, &mut self.buf);
        //let len = c.to_pulsedata(&mut self.buf[self.offset..]);
        self.offset += len;
    }

    pub fn get(&self, index: usize) -> Option<u16> {
        self.buf.get(index).cloned()
    }

    pub fn buffer(&self) -> &[u16] {
        &self.buf[..self.offset]
    }
}

impl<'a, Protocol: InfraredSender> IntoIterator for &'a PulsedataBuffer<Protocol> {
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
