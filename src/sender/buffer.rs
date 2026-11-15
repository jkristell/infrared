use crate::sender::{ProtocolEncoder, micros_to_ticks};

/// Pulse train in ticks, preceded by a zero length entry that starts the first mark
pub(crate) struct PulseBuffer<const S: usize> {
    buf: [u32; S],
    len: usize,
}

impl<const S: usize> PulseBuffer<S> {
    pub fn new() -> Self {
        Self {
            buf: [0; S],
            len: 0,
        }
    }

    pub fn reset(&mut self) {
        self.len = 0;
    }

    pub fn load<Proto: ProtocolEncoder>(&mut self, cmd: &Proto::Cmd, resolution: u32) {
        const {
            assert!(
                S > Proto::MAX_PULSES,
                "sender buffer too small for this protocol"
            )
        };

        self.buf[0] = 0;
        let n = Proto::encode(cmd, &mut self.buf[1..]);
        for pulse in &mut self.buf[1..=n] {
            *pulse = micros_to_ticks(*pulse, resolution).max(1);
        }
        self.len = n + 1;
    }

    /// Get the pulse length at `index`, if it is part of the loaded pulse train
    pub fn get(&self, index: usize) -> Option<u32> {
        self.buffer().get(index).copied()
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buf[..self.len]
    }
}
