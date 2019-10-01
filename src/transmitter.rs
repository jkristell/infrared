
#[derive(Debug)]
pub enum TransmitterState {
    /// Transmitter is ready for transmitting
    Idle,
    /// Transmitting
    Transmit(bool),
    /// Error state
    Err,
}

pub trait Transmitter<CMD> {
    /// Load command into transmitter
    fn load(&mut self, cmd: CMD);
    /// Step the transfer loop
    fn step(&mut self, ts: u32) -> TransmitterState;
    /// Reset the transmitter
    fn reset(&mut self);
}

#[cfg(feature = "embedded-hal")]
pub trait PwmTransmitter<CMD>: Transmitter<CMD> {
    /// Step the transmit loop and output on `pwm`
    fn pwmstep<PWMPIN, DUTY>(&mut self, ts: u32, pwm: &mut PWMPIN) -> TransmitterState
    where
        PWMPIN: embedded_hal::PwmPin<Duty=DUTY>,
    {
        let state = self.step(ts);
        match state {
            TransmitterState::Transmit(true) => pwm.enable(),
            _ => pwm.disable(),
        }
        state
    }
}
