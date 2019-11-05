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

    macro_rules! impl_receiver {
    ($ty:ident, [ $( ($N:ident, $P:ident, $C:ident, $E:ident) ),* ]) =>
    {
    pub struct $ty<PIN, $( $P ),* > {
        pin: PIN,
        $( $N : $P ),*
    }

    impl<PIN, PINERR, $( $P, $C, $E ),* > $ty <PIN, $( $P ),* >
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

    impl_receiver!(Receiver1, [
                (recv1, RECV1, CMD1, CMDERR1)
            ]);

    impl_receiver!(Receiver2, [
                (recv1, RECV1, CMD1, CMDERR1),
                (recv2, RECV2, CMD2, CMDERR2)
            ]);

    impl_receiver!(Receiver3, [
                (recv1, RECV1, CMD1, CMDERR1),
                (recv2, RECV2, CMD2, CMDERR2),
                (recv3, RECV3, CMD3, CMDERR3)
            ]);

    impl_receiver!(Receiver4, [
                (recv1, RECV1, CMD1, CMDERR1),
                (recv2, RECV2, CMD2, CMDERR2),
                (recv3, RECV3, CMD3, CMDERR3),
                (recv4, RECV4, CMD4, CMDERR4)
            ]);
}

#[cfg(feature = "protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

