//! Embedded-hal based Receivers

use embedded_hal::digital::v2::InputPin;

use crate::recv::{self, InfraredReceiver};
#[cfg(feature = "remotes")]
use crate::remotecontrol::{AsButton, Button, RemoteControl};

/// Event driven embedded-hal receiver
pub struct EventReceiver<Protocol: InfraredReceiver, Pin> {
    recv: crate::recv::EventReceiver<Protocol>,
    pin: Pin,
}

impl<Protocol, Pin, PinErr> EventReceiver<Protocol, Pin>
where
    Protocol: InfraredReceiver,
    Pin: InputPin<Error = PinErr>,
{
    /// Create a new EventReceiver
    /// `pin`: The Inputpin connected to the receiver,
    /// `resolution`: Resolution of the clock used
    pub fn new(pin: Pin, resolution: u32) -> Self {
        Self {
            recv: crate::recv::EventReceiver::new(resolution),
            pin,
        }
    }

    /// Destroy Receiver and hand back pin
    pub fn destroy(self) -> Pin {
        self.pin
    }

    /// Borrow pin mutable
    pub fn pin(&mut self) -> &mut Pin {
        &mut self.pin
    }

    /// Tell the receiver to read the new pin value and update the receiver state machine
    ///
    /// Returns Ok(None) until a command is detected
    #[inline(always)]
    pub fn update(&mut self, dt: u32) -> Result<Option<Protocol::Cmd>, PinErr> {
        let pinval = self.pin.is_low()?;

        match self.recv.update(pinval, dt) {
            Ok(cmd) => Ok(cmd),
            Err(_err) => Ok(None),
        }
    }
}

/// Periodic and polled Embedded hal Receiver
///
/// The poll methods should be called periodically for this receiver to work
pub struct PollReceiver<Protocol: InfraredReceiver, PIN> {
    /// The receiver state machine
    recv: recv::PollReceiver<Protocol>,
    /// Input pin
    pin: PIN,
    /// Internal sample counter
    counter: u32,
}

impl<Protocol, Pin, PinErr> PollReceiver<Protocol, Pin>
where
    Protocol: InfraredReceiver,
    Pin: InputPin<Error=PinErr>,
{
    /// Create a new PollReceiver
    /// `pin` : The gpio pin the hw is connected to
    /// `samplerate` : Rate of which you intend to call poll.
    pub fn new(pin: Pin, samplerate: u32) -> Self {
        Self {
            recv: recv::PollReceiver::new(samplerate),
            pin,
            counter: 0,
        }
    }

    pub fn destroy(self) -> Pin {
        self.pin
    }

    pub fn poll(&mut self) -> Result<Option<Protocol::Cmd>, PinErr> {
        let pinval = self.pin.is_low()?;

        self.counter = self.counter.wrapping_add(1);

        match self.recv.poll(pinval, self.counter) {
            Ok(cmd) => Ok(cmd),
            Err(_err) => Ok(None),
        }
    }

    #[cfg(feature = "remotes")]
    pub fn poll_button<RC: RemoteControl<Cmd = Protocol::Cmd>>(
        &mut self,
    ) -> Result<Option<Button>, PinErr>
    where
        Protocol::Cmd: AsButton,
    {
        self.poll().map(|cmd| cmd.and_then(RC::decode))
    }
}

macro_rules! multireceiver {
    (
        $(#[$outer:meta])*
        $name:ident, [ $( ($N:ident, $P:ident) ),* ]
    ) => {

    $(#[$outer])*
    pub struct $name<$( $P: InfraredReceiver ),* , PIN> {
        pin: PIN,
        counter: u32,
        $( $N : recv::PollReceiver<$P> ),*
    }

    impl<PIN, PINERR, $( $P ),* > $name <$( $P ),* , PIN>
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: InfraredReceiver),*,
    {
        pub fn new(pin: PIN, samplerate: u32) -> Self {
            Self {
                pin,
                counter: 0,
                $( $N: recv::PollReceiver::new(samplerate)),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn poll(&mut self) -> Result<( $( Option<$P::Cmd>),*), PINERR> {
            let pinval = self.pin.is_low()?;
            self.counter = self.counter.wrapping_add(1);

            Ok(($(
                match self.$N.poll(pinval, self.counter) {
                    Ok(cmd) => cmd,
                    Err(_err) => None,
                }
            ),* ))
        }
    }
};
}

multireceiver!(
    /// Receiver for two protocols
    PeriodicReceiver2,
    [(r1, R1), (r2, R2)]
);

multireceiver!(
    /// Receiver for three protocols
    PeriodicReceiver3,
    [(r1, R1), (r2, R2), (r3, R3)]
);

multireceiver!(
    /// Receiver for four protocols
    PeriodicReceiver4,
    [(r1, R1), (r2, R2), (r3, R3), (r4, R4)]
);

multireceiver!(
    /// Receiver for five protocols
    PeriodicReceiver5,
    [(r1, R1), (r2, R2), (r3, R3), (r4, R4), (r5, R5)]
);
