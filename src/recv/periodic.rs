//! Period reciever
//!
use crate::recv::{Error, EventReceiver, InfraredReceiver};

/// Receiver to use with periodic polling
pub struct PeriodicReceiver<Protocol>
where
    Protocol: InfraredReceiver,
{
    pub recv: EventReceiver<Protocol>,
    /// Last seen edge
    edge: bool,
    /// Seen at
    last: u32,
}

impl<Protocol: InfraredReceiver> PeriodicReceiver<Protocol> {
    pub fn new(samplerate: u32) -> Self {
        Self {
            recv: EventReceiver::new(samplerate),
            edge: false,
            last: 0,
        }
    }

    pub fn poll(&mut self, edge: bool, ts: u32) -> Result<Option<Protocol::Cmd>, Error> {
        if self.edge == edge {
            return Ok(None);
        }

        let dt = ts.wrapping_sub(self.last);

        self.last = ts;
        self.edge = edge;
        self.recv.update(edge, dt)
    }

    pub fn reset(&mut self) {
        self.recv.reset()
    }
}
