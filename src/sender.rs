//! Transmitter state machine
//!

#[derive(Debug)]
/// Sender state
pub enum State {
    /// Sender is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error
    Error,
}

/// Sender
pub trait Sender<CMD> {
    /// Load command into the sender
    fn load(&mut self, cmd: CMD);
    /// Step the transfer loop
    fn step(&mut self, ts: u32) -> State;
    /// Reset the transmitter
    fn reset(&mut self);
}

#[cfg(feature = "embedded-hal")]
/// Embedded hal IR Sender
pub trait PwmPinSender<CMD>: Sender<CMD> {
    /// Step the transmit loop and output on `pwm`
    fn step_pwm<PWMPIN, DUTY>(&mut self, ts: u32, pwm: &mut PWMPIN) -> State
    where
        PWMPIN: embedded_hal::PwmPin<Duty = DUTY>,
    {
        let state = self.step(ts);
        match state {
            State::Transmit(true) => pwm.enable(),
            _ => pwm.disable(),
        }
        state
    }
}
