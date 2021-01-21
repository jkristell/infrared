use crate::sender::{PulseSender, State};
use crate::PulseLengths;
use core::convert::Infallible;

pub struct HalSender<PWMPIN, DUTY>
where
    PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pts: PulseSender,
    pin: PWMPIN,
    pub counter: u32,
}

impl<'a, PWMPIN, DUTY> HalSender<PWMPIN, DUTY>
where
    PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
{
    pub fn new(samplerate: u32, pin: PWMPIN) -> Self {
        Self {
            pts: PulseSender::new(samplerate),
            pin,
            counter: 0,
        }
    }

    pub fn load<C: PulseLengths>(&mut self, cmd: &C) -> nb::Result<(), Infallible> {
        if self.pts.state == State::Idle {
            self.pts.load_command(cmd);
            self.counter = 0;
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Get a reference to the data
    pub fn buffer(&self) -> &[u16] {
        &self.pts.buffer()
    }

    /// Method to be called periodically to update the pwm output
    pub fn tick(&mut self) {
        let state = self.pts.tick(self.counter);
        self.counter = self.counter.wrapping_add(1);

        match state {
            State::Transmit(true) => self.pin.enable(),
            State::Transmit(false) => self.pin.disable(),
            State::Idle => self.pin.disable(),
            State::Error => self.pin.disable(),
        };
    }
}
