//! Embedded-hal based Receiver types

use embedded_hal::digital::v2::InputPin;

use crate::recv::{self, ReceiverSM};
use crate::remotecontrol::AsRemoteControlButton;
use crate::{Button, RemoteControl};

/// Event driven Hal receiver
pub struct EventReceiver<PROTOCOL, PIN> {
    recv: recv::EventReceiver<PROTOCOL>,
    pub pin: PIN,
}

impl<PIN, PINERR, PROTOCOL> EventReceiver<PROTOCOL, PIN>
where
    PROTOCOL: ReceiverSM,
    PIN: InputPin<Error = PINERR>,
{
    /// Create a new EventReceiver
    /// `pin`: The Inputpin connected to the receiver,
    /// `samplerate`: Sample rate of the receiver
    pub fn new(pin: PIN, samplerate: u32) -> Self {
        Self {
            recv: recv::EventReceiver::new(samplerate),
            pin,
        }
    }

    /// Destroy Receiver and hand back pin
    pub fn destroy(self) -> PIN {
        self.pin
    }

    /// Tell the receiver to read the new pin value and update the receiver state machine
    ///
    /// Returns Ok(None) until a command is detected
    #[inline(always)]
    pub fn edge_event(&mut self, dt: u32) -> Result<Option<PROTOCOL::Cmd>, PINERR> {
        let pinval = self.pin.is_low()?;

        match self.recv.edge_event(pinval, dt) {
            Ok(cmd) => Ok(cmd),
            Err(_err) => Ok(None),
        }
    }
}

/// Periodic and polled Embedded hal Receiver
///
/// The poll methods should be called periodically for this receiver to work
pub struct PeriodicReceiver<PROTOCOL, PIN> {
    /// The receiver state machine
    recv: recv::PeriodicReceiver<PROTOCOL>,
    /// Input pin
    pin: PIN,
    /// Internal sample counter
    counter: u32,
}

impl<PIN, PINERR, PROTOCOL> PeriodicReceiver<PROTOCOL, PIN>
where
    PROTOCOL: ReceiverSM,
    PIN: InputPin<Error = PINERR>,
{
    /// Create a new PeriodicReceiver
    /// `pin` : The gpio pin the hw is connected to
    /// `samplerate` : Rate of which you intend to call poll.
    pub fn new(pin: PIN, samplerate: u32) -> Self {
        Self {
            recv: recv::PeriodicReceiver::new(samplerate),
            pin,
            counter: 0,
        }
    }

    pub fn destroy(self) -> PIN {
        self.pin
    }

    pub fn poll(&mut self) -> Result<Option<PROTOCOL::Cmd>, PINERR> {
        let pinval = self.pin.is_low()?;

        self.counter = self.counter.wrapping_add(1);

        match self.recv.poll(pinval, self.counter) {
            Ok(cmd) => Ok(cmd),
            Err(_err) => Ok(None),
        }
    }

    #[cfg(feature = "remotes")]
    pub fn poll_button<RC: RemoteControl<Cmd = PROTOCOL::Cmd>>(
        &mut self,
    ) -> Result<Option<Button>, PINERR>
    where
        <PROTOCOL as ReceiverSM>::Cmd: AsRemoteControlButton,
    {
        self.poll().map(|cmd| cmd.and_then(RC::decode))
    }
}

macro_rules! multireceiver {
    (
        $(#[$outer:meta])*
        $name:ident, [ $( ($N:ident, $P:ident, $C:ident) ),* ]
    ) => {

    $(#[$outer])*
    pub struct $name<$( $P ),* , PIN> {
        pin: PIN,
        counter: u32,
        $( $N : recv::PeriodicReceiver<$P> ),*
    }

    impl<PIN, PINERR, $( $P, $C ),* > $name <$( $P ),* , PIN>
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: ReceiverSM<Cmd = $C>),*,
        //$( $C: crate::Command ),*,
    {
        pub fn new(pin: PIN, samplerate: u32) -> Self {
            Self {
                pin,
                counter: 0,
                $( $N: recv::PeriodicReceiver::new(samplerate)),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn poll(&mut self) -> Result<( $( Option<$C>),*), PINERR> {
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
    [(recv1, RECV1, CMD1), (recv2, RECV2, CMD2)]
);

multireceiver!(
    /// Receiver for three protocols
    PeriodicReceiver3,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3)
    ]
);

multireceiver!(
    /// Receiver for four protocols
    PeriodicReceiver4,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4)
    ]
);

multireceiver!(
    /// Receiver for five protocols
    PeriodicReceiver5,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4),
        (recv5, RECV5, CMD5)
    ]
);
