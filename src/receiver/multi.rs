#[cfg(feature = "embedded-hal")]
use embedded_hal::digital::InputPin;

use crate::{
    cmd::AnyCommand,
    receiver::{time::InfraMonotonic, DecoderBuilder, NoPin, Receiver},
};

/// Multi Receiver
pub struct MultiReceiver<
    const N: usize,
    Receivers: ReceiverWrapper<N, Time>,
    Input,
    Time: InfraMonotonic = u32,
> {
    receivers: Receivers::Receivers,
    input: Input,
}

impl<const N: usize, Receivers: ReceiverWrapper<N, Mono>, Input, Mono: InfraMonotonic>
    MultiReceiver<N, Receivers, Input, Mono>
{
    pub fn new(res: u32, input: Input) -> Self {
        MultiReceiver {
            input,
            receivers: Receivers::make(res),
        }
    }

    pub fn event_generic(&mut self, dt: Mono::Duration, edge: bool) -> [Option<AnyCommand>; N] {
        Receivers::event(&mut self.receivers, dt, edge)
    }

    pub fn event_generic_iter(
        &mut self,
        dt: Mono::Duration,
        flank: bool,
    ) -> impl Iterator<Item = AnyCommand> {
        let arr = self.event_generic(dt, flank);
        arr.into_iter().flatten()
    }
}

#[cfg(feature = "embedded-hal")]
impl<const N: usize, Receivers, Pin: InputPin, Mono: InfraMonotonic>
    MultiReceiver<N, Receivers, Pin, Mono>
where
    Receivers: ReceiverWrapper<N, Mono>,
{
    pub fn event(&mut self, dt: Mono::Duration) -> Result<[Option<AnyCommand>; N], Pin::Error> {
        let edge = self.input.is_low()?;
        Ok(self.event_generic(dt, edge))
    }

    pub fn event_iter(
        &mut self,
        dt: Mono::Duration,
    ) -> Result<impl Iterator<Item = AnyCommand>, Pin::Error> {
        let arr = self.event(dt)?;
        Ok(arr.into_iter().flatten())
    }

    /// Get a reference to the Pin
    pub fn pin(&self) -> &Pin {
        &self.input
    }

    /// Get a mut ref to the Pin
    pub fn pin_mut(&mut self) -> &mut Pin {
        &mut self.input
    }

    /// Drop the receiver and release the pin
    pub fn release(self) -> Pin {
        self.input
    }
}

pub trait ReceiverWrapper<const N: usize, Mono: InfraMonotonic> {
    type Receivers;

    fn make(res: u32) -> Self::Receivers;

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, flank: bool) -> [Option<AnyCommand>; N];
}

macro_rules! impl_receiver_wrapper {
    ($N:literal; $($P:ident),+; $($idx:tt),+) => {
        impl<$($P,)+ Mono: InfraMonotonic> ReceiverWrapper<$N, Mono> for ($($P,)+)
        where
            $($P: DecoderBuilder<Mono>,)+
            $($P::Cmd: Into<AnyCommand>,)+
        {
            type Receivers = ($(Receiver<$P, NoPin, Mono>,)+);

            fn make(res: u32) -> Self::Receivers {
                ($(Receiver::<$P, NoPin, Mono>::new(res),)+)
            }

            fn event(
                rs: &mut Self::Receivers,
                dt: Mono::Duration,
                edge: bool,
            ) -> [Option<AnyCommand>; $N] {
                [$(rs.$idx.event(dt, edge).unwrap_or_default().map(Into::into),)+]
            }
        }
    };
}

impl_receiver_wrapper!(2; P1, P2; 0, 1);
impl_receiver_wrapper!(3; P1, P2, P3; 0, 1, 2);
impl_receiver_wrapper!(4; P1, P2, P3, P4; 0, 1, 2, 3);
impl_receiver_wrapper!(5; P1, P2, P3, P4, P5; 0, 1, 2, 3, 4);
impl_receiver_wrapper!(6; P1, P2, P3, P4, P5, P6; 0, 1, 2, 3, 4, 5);
