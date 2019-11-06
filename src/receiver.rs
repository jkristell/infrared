use crate::ProtocolId;

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum ReceiverState<CMD, ERR> {
    Idle,
    Receiving,
    Done(CMD),
    Error(ERR),
    Disabled,
}

/// Receiver trait
pub trait Receiver {
    /// The resulting command type
    type Cmd;
    /// Receive Error
    type Err;
    /// Protocol id
    const PROTOCOL_ID: ProtocolId;

    /// Sample
    fn sample(&mut self, pinval: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Sample on known edge
    fn sample_edge(&mut self, rising: bool, sampletime: u32) -> ReceiverState<Self::Cmd, Self::Err>;
    /// Reset receiver
    fn reset(&mut self);
    /// Disable receiver
    fn disable(&mut self);
}


#[cfg(feature = "embedded-hal")]
pub mod hal {
    use embedded_hal::digital::v2::InputPin;
    use crate::ReceiverState;

    macro_rules! create_receiver {
    ($name:ident, [ $( ($N:ident, $P:ident, $C:ident, $E:ident) ),* ]) =>
    {
    /// HAL receiver
    pub struct $name<PIN, $( $P ),* > {
        pin: PIN,
        $( $N : $P ),*
    }

    impl<PIN, PINERR, $( $P, $C, $E ),* > $name <PIN, $( $P ),* >
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: crate::Receiver<Cmd = $C, Err = $E> ),*
    {
        pub fn new(pin: PIN, $( $N : $P ),* ) -> Self {
            Self {
                pin,
                $( $N ),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        #[allow(unused_parens)]
        pub fn step(&mut self, ts: u32) -> Result<( $( Option<$C>),*), PINERR> {
            let pinval = self.pin.is_low()?;

            Ok(($(
            match self.$N.sample(pinval, ts) {
                ReceiverState::Done(cmd) => {
                    self.$N.reset();
                    Some(cmd)
                },
                ReceiverState::Error(_) => {
                    self.$N.reset();
                    None
                }
                _ => None,
            }
            ),* ))
        }
    }

    };
}

    create_receiver!(HalReceiver, [
                (recv1, RECV1, CMD1, RECVERR1)
            ]);

    create_receiver!(HalReceiver2, [
                (recv1, RECV1, CMD1, RECVERR1),
                (recv2, RECV2, CMD2, RECVERR2)
            ]);

    create_receiver!(HalReceiver3, [
                (recv1, RECV1, CMD1, RECVERR1),
                (recv2, RECV2, CMD2, RECVERR2),
                (recv3, RECV3, CMD3, RECVERR3)
            ]);

    create_receiver!(HalReceiver4, [
                (recv1, RECV1, CMD1, RECVERR1),
                (recv2, RECV2, CMD2, RECVERR2),
                (recv3, RECV3, CMD3, RECVERR3),
                (recv4, RECV4, CMD4, RECVERR4)
            ]);
}

#[cfg(feature = "protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

