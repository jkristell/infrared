use crate::receiver::{
    DecoderStateMachine, DecodingError, DefaultInput, Error, Event, PinInput, Poll, Receiver,
};
#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::v2::InputPin;

pub struct MultiReceiver<P1, P2, IN>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
{
    r1: Receiver<P1, Event, DefaultInput>,
    r2: Receiver<P2, Event, DefaultInput>,
    input: IN,
}

#[cfg(feature = "embedded-hal")]
impl<P1, P2, PIN: InputPin> MultiReceiver<P1, P2, PinInput<PIN>>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
{
    pub fn event(
        &mut self,
        dt: usize,
    ) -> Result<(Option<P1::Cmd>, Option<P2::Cmd>), Error<PIN::Error>> {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        self.event_generic(dt, edge).map_err(Into::into)
    }
}

impl<P1, P2, IN> MultiReceiver<P1, P2, IN>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
{
    pub fn event_generic(
        &mut self,
        dt: usize,
        edge: bool,
    ) -> Result<(Option<P1::Cmd>, Option<P2::Cmd>), DecodingError> {
        Ok((self.r1.event(dt, edge)?, self.r2.event(dt, edge)?))
    }
}

pub struct MultiReceiver5<P1, P2, P3, P4, P5, MD, IN>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P5: DecoderStateMachine,
{
    r1: Receiver<P1, Event, DefaultInput>,
    r2: Receiver<P2, Event, DefaultInput>,
    r3: Receiver<P3, Event, DefaultInput>,
    r4: Receiver<P4, Event, DefaultInput>,
    r5: Receiver<P5, Event, DefaultInput>,
    data: MD,
    input: IN,
}

#[cfg(feature = "embedded-hal")]
impl<P1, P2, P3, P4, P5, PIN: InputPin> MultiReceiver5<P1, P2, P3, P4, P5, Poll, PinInput<PIN>>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P5: DecoderStateMachine,
{
    #[inline(always)]
    pub fn poll(
        &mut self,
    ) -> Result<
        (
            Option<P1::Cmd>,
            Option<P2::Cmd>,
            Option<P3::Cmd>,
            Option<P4::Cmd>,
            Option<P5::Cmd>,
        ),
        Error<PIN::Error>,
    > {
        let edge = self.input.0.is_low().map_err(Error::Hal)?;

        self.data.clock = self.data.clock.wrapping_add(1);

        if edge == self.data.edge {
            return Ok((None, None, None, None, None));
        }

        let ds = self.data.clock.wrapping_sub(self.data.last_edge);

        self.data.edge = edge;
        self.data.last_edge = self.data.clock;

        self.event_generic(ds, edge).map_err(Into::into)
    }
}

impl<P1, P2, P3, P4, P5, MD, IN> MultiReceiver5<P1, P2, P3, P4, P5, MD, IN>
where
    P1: DecoderStateMachine,
    P2: DecoderStateMachine,
    P3: DecoderStateMachine,
    P4: DecoderStateMachine,
    P5: DecoderStateMachine,
    MD: Default,
{
    pub fn new(resolution: usize, input: IN) -> Self {
        MultiReceiver5 {
            r1: Receiver::with_input(resolution, DefaultInput),
            r2: Receiver::with_input(resolution, DefaultInput),
            r3: Receiver::with_input(resolution, DefaultInput),
            r4: Receiver::with_input(resolution, DefaultInput),
            r5: Receiver::with_input(resolution, DefaultInput),
            data: MD::default(),
            input,
        }
    }

    pub fn event_generic(
        &mut self,
        dt: usize,
        edge: bool,
    ) -> Result<
        (
            Option<P1::Cmd>,
            Option<P2::Cmd>,
            Option<P3::Cmd>,
            Option<P4::Cmd>,
            Option<P5::Cmd>,
        ),
        DecodingError,
    > {
        Ok((
            self.r1.event(dt, edge).ok().flatten(),
            self.r2.event(dt, edge).ok().flatten(),
            self.r3.event(dt, edge).ok().flatten(),
            self.r4.event(dt, edge).ok().flatten(),
            self.r5.event(dt, edge).ok().flatten(),
        ))
    }
}

/*
macro_rules! multireceiver {
    (
        $(#[$outer:meta])*
        $name:ident, [ $( ($N:ident, $P:ident) ),* ]
    ) => {

    $(#[$outer])*
    pub struct $name<$( $P: DecoderStateMachine ),* , IN> {
        input: IN,
        $( $N : Receiver<$P, Evented, GenericInput> ),*
    }

    /*
    impl<PIN, PINERR, $( $P ),* > $name <$( $P ),* , PIN>
    where
        PIN: InputPin<Error = PINERR>,
        $( $P: InfraredReceiver),*,
    {
        pub fn new(pin: PIN, samplerate: usize) -> Self {
            Self {
                pin,
                counter: 0,
                $( $N: receiver::PollReceiver::new(samplerate)),*,
            }
        }

        pub fn destroy(self) -> PIN {
            self.pin
        }

        pub fn poll(&mut self) -> Result<( $( Option<$P::Cmd>),*), Error<PINERR>> {
            let pinval = self.pin.is_low()
                .map_err(|err| Error::Hal(err))?;

            self.counter = self.counter.wrapping_add(1);

            Ok(($(
                match self.$N.poll(pinval, self.counter) {
                    Ok(cmd) => cmd,
                    Err(_err) => None,
                }
            ),* ))
        }
    }
     */
};
}

multireceiver!(MultiReceiver3, [(r1, R1), (r2, R2)]);

*/
