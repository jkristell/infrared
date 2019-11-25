use crate::ProtocolId;


/// Receiver state machine
pub trait ReceiverStateMachine {
    /// Protocol id
    const ID: ProtocolId;
    /// The resulting command type
    type Cmd;

    // Create
    fn for_samplerate(samplerate: u32) -> Self;

    /// Add event to state machine
    fn event(&mut self, edge: bool, time: u32) -> ReceiverState<Self::Cmd>;
    /// Reset receiver
    fn reset(&mut self);
}

#[derive(PartialEq, Eq, Copy, Clone)]
/// Protocol decoder state
pub enum ReceiverState<CMD> {
    Idle,
    Receiving,
    Done(CMD),
    Error(ReceiverError),
    Disabled,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ReceiverError {
    Address(u32),
    Data(u32),
    Other(u32),
}

#[cfg(feature = "embedded-hal")]
pub mod ehal {
    use crate::protocols::sbp::SbpReceiver;
    use crate::{ReceiverStateMachine, ReceiverState};
    use crate::receiver::ReceiverState::Receiving;
    use crate::remotes::RemoteControl;

    /// Receiver Hal
    pub trait ReceiverHal<PIN, PINERR, CMD>
        where
            CMD: crate::Command,
    {
        /// Sample
        fn sample(&mut self, sampletime: u32) -> Result<Option<CMD>, PINERR>;

        #[cfg(feature = "remotes")]
        fn sample_remote<REMOTE>(&mut self, sampletime: u32) -> Result<Option<REMOTE::Button>, PINERR>
            where
                REMOTE: crate::remotes::RemoteControl<Command=CMD>;

        /// Disable receiver
        fn disable(&mut self);
    }



    pub struct GenericHalReceiver<PIN, SM> {
        sm: SM,
        pin: PIN,
        pinval: bool,
    }

    impl<CMD, PIN, SM: ReceiverStateMachine<Cmd=CMD>> GenericHalReceiver<PIN, SM> {

        pub fn new(pin: PIN, sm: SM) -> Self {
            Self {
                sm,
                pin,
                pinval: false,
            }
        }

        pub fn new_test(pin: PIN) -> Self {
            Self {
                sm: SM::for_samplerate(40_000),
                pin,
                pinval: false,
            }
        }
    }

    impl<SM, PIN, PINERR, CMD> ReceiverHal<PIN, PINERR, CMD> for GenericHalReceiver<PIN, SM>
        where
            CMD: crate::Command,
            SM: ReceiverStateMachine<Cmd=CMD>,
            PIN: embedded_hal::digital::v2::InputPin<Error=PINERR>,
    {
        fn sample(&mut self, sampletime: u32) -> Result<Option<CMD>, PINERR> {
            let pinval = self.pin.is_low()?;

            if self.pinval != pinval {
                let r = self.sm.event(pinval, sampletime);

                if let ReceiverState::Done(cmd) = r {
                    self.sm.reset();
                    return Ok(Some(cmd));
                }

                if let ReceiverState::Error(_err) = r {
                    self.sm.reset();
                }

                self.pinval = pinval;
            }

            Ok(None)
        }

        #[cfg(feature = "remotes")]
        fn sample_remote<REMOTE>(&mut self, sampletime: u32) -> Result<Option<REMOTE::Button>, PINERR>
            where
                REMOTE: crate::remotes::RemoteControl<Command=CMD>,
        {
            self
                .sample(sampletime)
                .map(|opt| opt.and_then(|cmd| REMOTE::decode_with_address(cmd)))
        }

        fn disable(&mut self) {
            unimplemented!()
        }
    }

}





/*

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

*/

#[cfg(feature = "protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

