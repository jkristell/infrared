//! Embedded-hal based PwmSender

use embedded_hal::pwm::SetDutyCycle;

use crate::sender::{Busy, DEFAULT_BUFFER_SIZE, ProtocolEncoder, TickSender, Status};

/// Embedded-hal `SetDutyCycle` based sender
///
/// Drives the infrared carrier by switching the duty cycle of a PWM pin
/// between zero and the configured on-duty. The PWM should be configured
/// to the carrier frequency of the protocol (typically ~38 kHz) and
/// [`tick`](PwmSender::tick) must be called periodically at `resolution` Hz.
pub struct PwmSender<Pwm, const S: usize = DEFAULT_BUFFER_SIZE> {
    pwm: Pwm,
    duty_on: u16,
    counter: u32,
    status: Status,
    buffer: TickSender<S>,
}

impl<Pwm, const S: usize> PwmSender<Pwm, S>
where
    Pwm: SetDutyCycle,
{
    /// Create a new PwmSender, ticked at `resolution` Hz, with a 25% carrier duty cycle
    pub fn new(pwm: Pwm, resolution: u32) -> Self {
        Self::with_duty_fraction(pwm, resolution, 1, 4)
    }

    /// Create a new PwmSender, ticked at `resolution` Hz, with a carrier duty cycle of `num / denom`
    ///
    /// Requires `denom != 0` and `num <= denom`.
    pub fn with_duty_fraction(pwm: Pwm, resolution: u32, num: u16, denom: u16) -> Self {
        let duty_on = duty_fraction(&pwm, num, denom);

        Self {
            pwm,
            duty_on,
            counter: 0,
            status: Status::Idle,
            buffer: TickSender::new(resolution),
        }
    }

    /// Load a command into the sender
    ///
    /// The transmission starts on the following [`tick`](PwmSender::tick).
    /// Fails if a transmission is already in progress, and fails to compile
    /// if the buffer is too small for the protocol.
    pub fn load<Proto>(&mut self, cmd: &Proto::Cmd) -> Result<(), Busy>
    where
        Proto: ProtocolEncoder,
    {
        if !self.buffer.is_idle() {
            return Err(Busy);
        }
        self.buffer.load::<Proto>(cmd);
        self.counter = 0;
        Ok(())
    }

    /// True if no transmission is in progress
    pub fn is_idle(&self) -> bool {
        self.buffer.is_idle()
    }

    /// The currently loaded pulse buffer
    pub fn buffer(&self) -> &[u32] {
        self.buffer.buffer()
    }

    /// Update the carrier output. To be called periodically at `resolution` Hz
    pub fn tick(&mut self) -> Result<Status, Pwm::Error> {
        let status = self.buffer.tick(self.counter);
        self.counter = self.counter.wrapping_add(1);

        // Only touch the pwm on state changes
        if status != self.status {
            if status.is_carrier_on() {
                self.pwm.set_duty_cycle(self.duty_on)?
            } else {
                self.pwm.set_duty_cycle_fully_off()?
            }
            self.status = status;
        }

        Ok(status)
    }

    /// Get a reference to the pwm pin
    pub fn pwm(&self) -> &Pwm {
        &self.pwm
    }

    /// Get a mut reference to the pwm pin
    pub fn pwm_mut(&mut self) -> &mut Pwm {
        &mut self.pwm
    }

    /// Release the pwm pin
    pub fn release(self) -> Pwm {
        self.pwm
    }
}

/// On duty of `pwm` for a duty cycle of `num / denom`
pub(crate) fn duty_fraction<Pwm: SetDutyCycle>(pwm: &Pwm, num: u16, denom: u16) -> u16 {
    debug_assert!(denom != 0 && num <= denom);
    (u32::from(pwm.max_duty_cycle()) * u32::from(num) / u32::from(denom.max(1))) as u16
}
