
pub mod receiver;
pub mod transmitter;

pub use receiver::{
    Rc5Receiver, Rc5Command
};


#[cfg(test)]
mod tests {
    use crate::rc5::Rc5Receiver;
    use crate::prelude::*;

    #[test]
    fn command() {

        let dists = [0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37];

        let mut recv = Rc5Receiver::new(40_000);
        let mut edge = false;
        let mut tot = 0;
        let mut state = ReceiverState::Idle;

        for dist in dists.iter() {
            edge = !edge;
            tot += dist;
            state = recv.sample_edge(edge, tot);
        }

        if let ReceiverState::Done(cmd) = state {
            assert_eq!(cmd.addr, 20);
            assert_eq!(cmd.cmd, 9);
        } else {
            assert!(false);
        }
    }
}

