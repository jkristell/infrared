use crate::{
    Level,
    protocol::Message,
    receiver::{
        time::InfraMonotonic, DecoderBuilder, DecodingError, EdgeDriven, Receiver, Timeout,
    },
};

/// Multi Receiver
///
/// Runs multiple protocol decoders on the same input signal. Hardware
/// independent, like [`Receiver`]. Combine it with
/// [`PinReceiver`](crate::receiver::PinReceiver) to let it read the edges
/// from an embedded-hal pin.
pub struct MultiReceiver<
    const N: usize,
    Receivers: ReceiverSet<N, Mono>,
    Mono: InfraMonotonic = u32,
> {
    receivers: Receivers::Receivers,
    prev_instant: Mono::Instant,
}

impl<const N: usize, Receivers: ReceiverSet<N, Mono>, Mono: InfraMonotonic>
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
    /// * `level`: The input level after the edge, see [`Level`]
    pub fn event(&mut self, dt: Mono::Duration, level: Level) -> [Option<Message>; N] {
        Receivers::event(&mut self.receivers, dt, level)
    }

    /// Update the receivers and get an iterator over the decoded commands
    pub fn event_iter(
        &mut self,
        dt: Mono::Duration,
        level: Level,
    ) -> impl Iterator<Item = Message> {
        self.event(dt, level).into_iter().flatten()
    }

    /// Update the receivers with an edge event at instant `t`
    ///
    /// The duration since the previous edge is tracked internally.
    pub fn event_instant(&mut self, t: Mono::Instant, level: Level) -> [Option<Message>; N] {
        let dt = Mono::checked_sub(t, self.prev_instant).unwrap_or(Mono::ZERO_DURATION);
        self.prev_instant = t;

        self.event(dt, level)
    }

    /// Notify the receivers that the input has been idle for at least
    /// [`idle_timeout`](MultiReceiver::idle_timeout)
    ///
    /// Discards any partially received frames.
    pub fn timeout(&mut self) -> [Option<Message>; N] {
        Receivers::timeout(&mut self.receivers)
    }

    /// Time without edges after which the input is considered idle
    ///
    /// The longest idle timeout of the protocols.
    pub fn idle_timeout(&self) -> Mono::Duration {
        Receivers::idle_timeout(&self.receivers)
    }
}

impl<const N: usize, Receivers, Mono> EdgeDriven for MultiReceiver<N, Receivers, Mono>
where
    Receivers: ReceiverSet<N, Mono>,
    Mono: InfraMonotonic,
{
    type Duration = Mono::Duration;
    type Instant = Mono::Instant;
    type Output = [Option<Message>; N];

    fn event_edge(
        &mut self,
        dt: Self::Duration,
        level: Level,
    ) -> Result<Self::Output, DecodingError> {
        Ok(self.event(dt, level))
    }

    fn event_edge_instant(
        &mut self,
        t: Self::Instant,
        level: Level,
    ) -> Result<Self::Output, DecodingError> {
        Ok(self.event_instant(t, level))
    }

    fn idle_timeout(&self) -> Self::Duration {
        MultiReceiver::idle_timeout(self)
    }
}

impl<const N: usize, Receivers, Mono> Timeout for MultiReceiver<N, Receivers, Mono>
where
    Receivers: ReceiverSet<N, Mono>,
    Mono: InfraMonotonic,
{
    type Output = [Option<Message>; N];

    fn timeout(&mut self) -> Result<Self::Output, DecodingError> {
        Ok(MultiReceiver::timeout(self))
    }
}

pub trait ReceiverSet<const N: usize, Mono: InfraMonotonic> {
    type Receivers;

    fn make(res: u32) -> Self::Receivers;

    fn event(rs: &mut Self::Receivers, dt: Mono::Duration, level: Level) -> [Option<Message>; N];

    fn timeout(rs: &mut Self::Receivers) -> [Option<Message>; N];

    fn idle_timeout(rs: &Self::Receivers) -> Mono::Duration;
}

macro_rules! impl_receiver_wrapper {
    ($N:literal; $($P:ident),+; $($idx:tt),+) => {
        impl<$($P,)+ Mono: InfraMonotonic> ReceiverSet<$N, Mono> for ($($P,)+)
        where
            $($P: DecoderBuilder<Mono>,)+
            $($P::Cmd: Into<Message>,)+
        {
            type Receivers = ($(Receiver<$P, Mono>,)+);

            fn make(res: u32) -> Self::Receivers {
                ($(Receiver::<$P, Mono>::new(res),)+)
            }

            fn event(
                rs: &mut Self::Receivers,
                dt: Mono::Duration,
                level: Level,
            ) -> [Option<Message>; $N] {
                [$(rs.$idx.event(dt, level).unwrap_or_default().map(Into::into),)+]
            }

            fn timeout(rs: &mut Self::Receivers) -> [Option<Message>; $N] {
                [$(rs.$idx.timeout().unwrap_or_default().map(Into::into),)+]
            }

            fn idle_timeout(rs: &Self::Receivers) -> Mono::Duration {
                let mut max = Mono::ZERO_DURATION;
                $(
                    let t = rs.$idx.idle_timeout();
                    if t > max {
                        max = t;
                    }
                )+
                max
            }
        }
    };
}

impl_receiver_wrapper!(2; P1, P2; 0, 1);
impl_receiver_wrapper!(3; P1, P2, P3; 0, 1, 2);
impl_receiver_wrapper!(4; P1, P2, P3, P4; 0, 1, 2, 3);
impl_receiver_wrapper!(5; P1, P2, P3, P4, P5; 0, 1, 2, 3, 4);
impl_receiver_wrapper!(6; P1, P2, P3, P4, P5, P6; 0, 1, 2, 3, 4, 5);
