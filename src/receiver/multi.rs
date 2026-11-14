use crate::{
    cmd::AnyCommand,
    receiver::{time::InfraMonotonic, DecoderBuilder, DecodingError, EdgeReceiver, Receiver},
};

/// Multi Receiver
///
/// Runs multiple protocol decoders on the same input signal. Hardware
/// independent, like [`Receiver`]. Combine it with
/// [`PinReceiver`](crate::receiver::PinReceiver) to let it read the edges
/// from an embedded-hal pin.
pub struct MultiReceiver<
    const N: usize,
    Receivers: ReceiverWrapper<N, Mono>,
    Mono: InfraMonotonic = u32,
> {
    receivers: Receivers::Receivers,
    prev_instant: Mono::Instant,
}

impl<const N: usize, Receivers: ReceiverWrapper<N, Mono>, Mono: InfraMonotonic>
    MultiReceiver<N, Receivers, Mono>
{
    /// Create a new MultiReceiver with the given clock resolution (in Hz)
    pub fn new(res: u32) -> Self {
        MultiReceiver {
            receivers: Receivers::make(res),
            prev_instant: Mono::ZERO_INSTANT,
        }
    }

    /// Update the receivers
    /// * `dt`: Duration since the previous edge
    /// * `edge`: true = start of an IR mark (the pin reading low on an active-low receiver module), false = start of a space
    pub fn event(&mut self, dt: Mono::Duration, edge: bool) -> [Option<AnyCommand>; N] {
        Receivers::event(&mut self.receivers, dt, edge)
    }

    /// Update the receivers and get an iterator over the decoded commands
    pub fn event_iter(
        &mut self,
        dt: Mono::Duration,
        edge: bool,
    ) -> impl Iterator<Item = AnyCommand> {
        self.event(dt, edge).into_iter().flatten()
    }

    /// Update the receivers with an edge event at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_instant(&mut self, t: Mono::Instant, edge: bool) -> [Option<AnyCommand>; N] {
        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        self.event(dt, edge)
    }
}

impl<const N: usize, Receivers, Mono> EdgeReceiver for MultiReceiver<N, Receivers, Mono>
where
    Receivers: ReceiverWrapper<N, Mono>,
    Mono: InfraMonotonic,
{
    type Duration = Mono::Duration;
    type Instant = Mono::Instant;
    type Output = [Option<AnyCommand>; N];

    fn event_edge(
        &mut self,
        dt: Self::Duration,
        edge: bool,
    ) -> Result<Self::Output, DecodingError> {
        Ok(self.event(dt, edge))
    }

    fn event_edge_instant(
        &mut self,
        t: Self::Instant,
        edge: bool,
    ) -> Result<Self::Output, DecodingError> {
        Ok(self.event_instant(t, edge))
    }
}

pub trait ReceiverWrapper<const N: usize, Mono: InfraMonotonic> {
    type Receivers;

    fn make(res: u32) -> Self::Receivers;

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, edge: bool) -> [Option<AnyCommand>; N];
}

macro_rules! impl_receiver_wrapper {
    ($N:literal; $($P:ident),+; $($idx:tt),+) => {
        impl<$($P,)+ Mono: InfraMonotonic> ReceiverWrapper<$N, Mono> for ($($P,)+)
        where
            $($P: DecoderBuilder<Mono>,)+
            $($P::Cmd: Into<AnyCommand>,)+
        {
            type Receivers = ($(Receiver<$P, Mono>,)+);

            fn make(res: u32) -> Self::Receivers {
                ($(Receiver::<$P, Mono>::new(res),)+)
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
