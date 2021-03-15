//! Embedded-hal based Sender

use crate::sender::{ProtocolEncoder, PulsedataSender, Status};

/// Embedded hal sender
pub struct Sender<PwmPin, const FREQ: usize, const BUFSIZE: usize> {
    pin: PwmPin,
    counter: usize,
    buffer: PulsedataSender<BUFSIZE>,
}

impl<PwmPin, PwmDuty, const F: usize, const S: usize> Sender<PwmPin, F, S>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{
    pub fn new(pin: PwmPin) -> Self {
        Self {
            pin,
            counter: 0,
            buffer: PulsedataSender::new(),
        }
    }

    pub fn load<Proto>(&mut self, cmd: &Proto::Cmd)
    where
        Proto: ProtocolEncoder<F>,
    {
        if self.buffer.status == Status::Idle {
            self.buffer.load_command::<Proto, F>(cmd);
            self.counter = 0;
        }
    }

    pub fn buffer(&self) -> &[usize] {
        &self.buffer.buffer()
    }

    /// Method to be called periodically to update the pwm output
    pub fn tick(&mut self) {
        let status = self.buffer.tick(self.counter);
        self.counter = self.counter.wrapping_add(1);

        match status {
            Status::Transmit(true) => self.pin.enable(),
            Status::Transmit(false) => self.pin.disable(),
            Status::Idle => self.pin.disable(),
            Status::Error => self.pin.disable(),
        };
    }
}
