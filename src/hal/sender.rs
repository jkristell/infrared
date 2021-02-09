//! Embedded-hal based Sender

use core::convert::Infallible;

use crate::send::{InfraredSender, PulsedataSender, Status, InfraredSenderState};
use crate::protocolid::InfraredProtocol;

/// Embedded hal sender
pub struct Sender<Protocol, PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
    Protocol: InfraredSender,
{
    pin: PwmPin,
    pub counter: u32,
    buffer: PulsedataSender<Protocol>,
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
            buffer: PulsedataSender::new(samplerate),
        }
    }

    pub fn load(&mut self, cmd: &Protocol::Cmd) -> nb::Result<(), Infallible> {
        if self.buffer.status == Status::Idle {
            self.buffer.load_command(cmd);
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

pub struct MultiSender<PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{
    pin: PwmPin,
    pub offs: u32,
    pub buf: [u16; 48],
}

impl<PwmPin, PwmDuty> MultiSender<PwmPin, PwmDuty>
where
    PwmPin: embedded_hal::PwmPin<Duty = PwmDuty>,
{

    pub fn new(samplerate: u32, pin: PwmPin) -> Self {
        Self {
            pin,
            offs: 0,
            buf: [0; 48],
        }
    }

    pub fn load<Protocol, ProtocolState>(&mut self, state: &mut ProtocolState, cmd: &Protocol::Cmd)
    where
        Protocol: InfraredSender<State = ProtocolState>,
        ProtocolState: InfraredSenderState,
    {
        Protocol::cmd_pulsedata(state, cmd, &mut self.buf[0..]);
    }
}


