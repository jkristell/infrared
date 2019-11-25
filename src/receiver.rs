use crate::{ProtocolId, Command};


/// Receiver state machine
pub trait ReceiverStateMachine {
    /// Protocol id
    const ID: ProtocolId;
    /// The resulting command type
    type Cmd: Command;

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
    use embedded_hal::digital::v2::InputPin;
    use crate::receiver::{ReceiverStateMachine, ReceiverState};

    pub struct HalReceiver<PIN, SM> {
        sm: SM,
        pin: PIN,
        pinval: bool,
    }

    impl<CMD, PIN, PINERR, SM> HalReceiver<PIN, SM>
    where
        CMD: crate::Command,
        SM: ReceiverStateMachine<Cmd=CMD>,
        PIN: InputPin<Error=PINERR>,
    {

        pub fn new_from_sm(pin: PIN, sm: SM) -> Self {
            Self {
                sm,
                pin,
                pinval: false,
            }
        }

        pub fn new(pin: PIN, samplerate: u32) -> Self {
            Self {
                sm: SM::for_samplerate(samplerate),
                pin,
                pinval: false,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }


        pub fn sample(&mut self, sample: u32) -> Result<Option<CMD>, PINERR> {
            let pinval = self.pin.is_low()?;

            if self.pinval != pinval {
                let r = self.sm.event(pinval, sample);

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
        pub fn sample_remote<REMOTE>(&mut self, sampletime: u32) -> Result<Option<REMOTE::Button>, PINERR>
        where
            REMOTE: crate::remotes::RemoteControl<Command=CMD>,
        {
            self
                .sample(sampletime)
                .map(|opt| opt.and_then(|cmd| REMOTE::decode_command(cmd)))
        }
    }

    macro_rules! create_receiver {
    ($name:ident, [ $( ($N:ident, $P:ident, $C:ident) ),* ]) =>
    {
    /// HAL receiver
    pub struct $name<PIN, $( $P ),* > {
        pin: PIN,
        pinval: bool,
        $( $N : $P ),*
    }

    impl<PIN, PINERR, $( $P, $C ),* > $name <PIN, $( $P ),* >
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: ReceiverStateMachine<Cmd = $C>),*,
        $( $C: crate::Command ),*,
    {
        pub fn new_from_sm(pin: PIN, $( $N : $P ),* ) -> Self {
            Self {
                pin,
                pinval: false,
                $( $N ),*,
            }
        }

        pub fn new(pin: PIN, samplerate: u32) -> Self {
            Self {
                pin,
                pinval: false,
                $( $N: $P::for_samplerate(samplerate)),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn step(&mut self, ts: u32) -> Result<( $( Option<$C>),*), PINERR> {
            let pinval = self.pin.is_low()?;

            if self.pinval != pinval {
                self.pinval = pinval;

                Ok(($(
                match self.$N.event(pinval, ts) {
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
            } else {
                Ok(Default::default())
            }
        }
    }

    };
}


    create_receiver!(HalReceiver2, [
                (recv1, RECV1, CMD1),
                (recv2, RECV2, CMD2)
            ]);

    create_receiver!(HalReceiver3, [
                (recv1, RECV1, CMD1),
                (recv2, RECV2, CMD2),
                (recv3, RECV3, CMD3)
            ]);

    create_receiver!(HalReceiver4, [
                (recv1, RECV1, CMD1),
                (recv2, RECV2, CMD2),
                (recv3, RECV3, CMD3),
                (recv4, RECV4, CMD4)
            ]);
}


#[cfg(feature = "protocol-dev")]
pub struct ReceiverDebug<STATE, EXTRA> {
    pub state: STATE,
    pub state_new: STATE,
    pub delta: u16,
    pub extra: EXTRA,
}

