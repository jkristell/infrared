use embedded_hal::digital::v2::InputPin;

use crate::{
    receiver::{ReceiverState, ReceiverStateMachine},
};


/// Embedded hal Receiever
pub struct InfraredReceiver<PIN, SM> {
    /// The State Machine
    sm: SM,
    /// The pin used
    pin: PIN,
    pinval: bool,
}

impl<CMD, PIN, PINERR, SM> InfraredReceiver<PIN, SM>
where
    CMD: crate::Command,
    SM: ReceiverStateMachine<Cmd = CMD>,
    PIN: InputPin<Error = PINERR>,
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
    pub fn sample_as_button<RC>(&mut self, sampletime: u32) -> Result<Option<RC::Button>, PINERR>
    where
        RC: crate::remotes::RemoteControl<Command = CMD>,
    {
        self.sample(sampletime)
            .map(|opt| opt.and_then(RC::decode))
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

create_receiver!(InfraredReceiver2, [(recv1, RECV1, CMD1), (recv2, RECV2, CMD2)]);

create_receiver!(
    InfraredReceiver3,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3)
    ]
);

create_receiver!(
    InfraredReceiver4,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4)
    ]
);
create_receiver!(
    InfraredReceiver5,
    [
        (recv1, RECV1, CMD1),
        (recv2, RECV2, CMD2),
        (recv3, RECV3, CMD3),
        (recv4, RECV4, CMD4),
        (recv5, RECV5, CMD5)
    ]
);
