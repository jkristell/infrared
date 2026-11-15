//! Async sender

use embedded_hal::pwm::SetDutyCycle;
use embedded_hal_async::delay::DelayNs;

use crate::sender::{DEFAULT_BUFFER_SIZE, ProtocolEncoder, hal::duty_fraction};

/// Async sender driving the carrier with a `SetDutyCycle` pwm
///
/// The pwm should be configured to the carrier frequency of the protocol
/// (typically ~38 kHz). Marks and spaces are timed with an embedded-hal-async
/// [`DelayNs`].
///
/// ```ignore
/// let mut sender = AsyncSender::embassy(pwm);
/// sender.send::<Nec>(&NecCommand { addr: 0, cmd: 12, repeat: false }).await?;
/// ```
pub struct AsyncSender<Pwm, D> {
    pwm: Pwm,
    delay: D,
    duty_on: u16,
}

impl<Pwm, D> AsyncSender<Pwm, D>
where
    Pwm: SetDutyCycle,
    D: DelayNs,
{
    /// Create a new AsyncSender with a 25% carrier duty cycle
    pub fn new(pwm: Pwm, delay: D) -> Self {
        Self::with_duty_fraction(pwm, delay, 1, 4)
    }

    /// Create a new AsyncSender with a carrier duty cycle of `num / denom`
    ///
    /// Requires `denom != 0` and `num <= denom`.
    pub fn with_duty_fraction(mut pwm: Pwm, delay: D, num: u16, denom: u16) -> Self {
        let duty_on = duty_fraction(&pwm, num, denom);
        // Best effort, the carrier is also turned off after every command
        let _ = pwm.set_duty_cycle_fully_off();
        Self {
            pwm,
            delay,
            duty_on,
        }
    }

    /// Send a command, returning when the last mark has been sent
    ///
    /// The command is encoded into a [`DEFAULT_BUFFER_SIZE`] stack buffer.
    /// Fails to compile if the protocol needs more pulses than that.
    pub async fn send<Proto>(&mut self, cmd: &Proto::Cmd) -> Result<(), Pwm::Error>
    where
        Proto: ProtocolEncoder,
    {
        const {
            assert!(
                DEFAULT_BUFFER_SIZE >= Proto::MAX_PULSES,
                "sender buffer too small for this protocol"
            )
        };

        let mut buf = [0u32; DEFAULT_BUFFER_SIZE];
        let len = Proto::encode(cmd, &mut buf);

        // Turns the carrier off even if the future is dropped mid-transmission
        let guard = CarrierOff(&mut self.pwm);
        for (i, &us) in buf[..len].iter().enumerate() {
            if i % 2 == 0 {
                guard.0.set_duty_cycle(self.duty_on)?;
            } else {
                guard.0.set_duty_cycle_fully_off()?;
            }
            self.delay.delay_us(us).await;
        }
        guard.finish()
    }

    /// Get a reference to the pwm
    pub fn pwm(&self) -> &Pwm {
        &self.pwm
    }

    /// Get a mut reference to the pwm
    pub fn pwm_mut(&mut self) -> &mut Pwm {
        &mut self.pwm
    }

    /// Release the pwm and the delay
    pub fn release(self) -> (Pwm, D) {
        (self.pwm, self.delay)
    }
}

#[cfg(feature = "embassy")]
impl<Pwm> AsyncSender<Pwm, embassy_time::Delay>
where
    Pwm: SetDutyCycle,
{
    /// Create an AsyncSender timed by embassy-time
    pub fn embassy(pwm: Pwm) -> Self {
        Self::new(pwm, embassy_time::Delay)
    }
}

struct CarrierOff<'a, Pwm: SetDutyCycle>(&'a mut Pwm);

impl<Pwm: SetDutyCycle> CarrierOff<'_, Pwm> {
    fn finish(self) -> Result<(), Pwm::Error> {
        let mut this = core::mem::ManuallyDrop::new(self);
        this.0.set_duty_cycle_fully_off()
    }
}

impl<Pwm: SetDutyCycle> Drop for CarrierOff<'_, Pwm> {
    fn drop(&mut self) {
        let _ = self.0.set_duty_cycle_fully_off();
    }
}
