//! Embedded-hal based Sender

use embedded_hal::pwm::SetDutyCycle;

use crate::sender::{ProtocolEncoder, PulsedataSender, Status};

/// Embedded-hal `SetDutyCycle` based sender
///
/// Drives the infrared carrier by switching the duty cycle of a PWM pin
/// between zero and the configured on-duty. The PWM should be configured
/// to the carrier frequency of the protocol (typically ~38 kHz) and
/// [`tick`](Sender::tick) must be called periodically at a rate of `FREQ` Hz.
pub struct Sender<Pwm, const FREQ: u32, const BUFSIZE: usize> {
    pwm: Pwm,
    duty_on: u16,
    counter: u32,
    status: Status,
    buffer: PulsedataSender<BUFSIZE>,
}

impl<Pwm, const FREQ: u32, const BUFSIZE: usize> Sender<Pwm, FREQ, BUFSIZE>
where
    Pwm: SetDutyCycle,
{
    /// Create a new Sender with a 25% carrier duty cycle
    pub fn new(pwm: Pwm) -> Self {
        Self::with_duty_fraction(pwm, 1, 4)
    }

    /// Create a new Sender with a carrier duty cycle of `num / denom`
    ///
    /// Requires `denom != 0` and `num <= denom`.
    pub fn with_duty_fraction(pwm: Pwm, num: u16, denom: u16) -> Self {
        debug_assert!(denom != 0 && num <= denom);
        let duty_on =
            (u32::from(pwm.max_duty_cycle()) * u32::from(num) / u32::from(denom.max(1))) as u16;

        Self {
            pwm,
            duty_on,
            counter: 0,
            status: Status::Idle,
            buffer: PulsedataSender::new(),
        }
    }

    /// Load a command into the sender
    ///
    /// The transmission starts on the following [`tick`](Sender::tick).
    /// Ignored if a transmission is already in progress.
    pub fn load<Proto>(&mut self, cmd: &Proto::Cmd)
    where
        Proto: ProtocolEncoder<FREQ>,
    {
        if self.buffer.status == Status::Idle {
            self.buffer.load_command::<Proto, FREQ>(cmd);
            self.counter = 0;
        }
    }

    /// True if no transmission is in progress
    pub fn is_idle(&self) -> bool {
        self.buffer.status == Status::Idle
    }

    /// The currently loaded pulse buffer
    pub fn buffer(&self) -> &[u32] {
        self.buffer.buffer()
    }

    /// Update the carrier output. To be called periodically at a rate of `FREQ` Hz
    pub fn tick(&mut self) -> Result<Status, Pwm::Error> {
        let status = self.buffer.tick(self.counter);
        self.counter = self.counter.wrapping_add(1);

        // Only touch the pwm on state changes
        if status != self.status {
            match status {
                Status::Transmit(true) => self.pwm.set_duty_cycle(self.duty_on)?,
                Status::Transmit(false) | Status::Idle | Status::Error => {
                    self.pwm.set_duty_cycle_fully_off()?
                }
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
