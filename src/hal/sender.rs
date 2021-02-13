//! Embedded-hal based Sender

use core::convert::Infallible;

use crate::send::{InfraredSender, PulsedataSender, Status, InfraredSenderState};

/// Embedded hal sender
pub struct Sender<Protocol, PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
    Protocol: InfraredSender,
{
    pin: PwmPin,
    counter: u32,
    state: Protocol::State,
    buffer: PulsedataSender,
}

impl<Protocol, PwmPin, PwmDuty> Sender<Protocol, PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
    Protocol: InfraredSender,
{
    pub fn new(samplerate: u32, pin: PwmPin) -> Self {
        Self {
            pin,
            counter: 0,
            buffer: PulsedataSender::new(),
            state: Protocol::sender_state(samplerate),
        }
    }

    pub fn load(&mut self, cmd: &Protocol::Cmd) -> nb::Result<(), Infallible> {
        if self.buffer.status == Status::Idle {
            self.buffer.load_command::<Protocol>(&self.state, cmd);
            self.counter = 0;
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    /// Get a reference to the data
    pub fn buffer(&self) -> &[u16] {
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

/// Sender without a predefined Protocol.
pub struct MultiSender<PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{
    pin: PwmPin,
    samplerate: u32,
    buffer: PulsedataSender,
    counter: u32,
}

impl<PwmPin, PwmDuty> MultiSender<PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{
    pub fn new(samplerate: u32, pin: PwmPin) -> Self {
        Self {
            samplerate,
            pin,
            buffer: PulsedataSender::new(),
            counter: 0,
        }
    }

    pub fn create_state<ProtocolState: InfraredSenderState>(&self) -> ProtocolState {
        ProtocolState::create(self.samplerate)
    }

    pub fn load<Protocol>(&mut self, state: &mut Protocol::State, cmd: &Protocol::Cmd)
    where
        Protocol: InfraredSender,
    {
        if self.buffer.status == Status::Idle {
            self.buffer.load_command::<Protocol>(state, cmd);
            self.counter = 0;
        }
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


