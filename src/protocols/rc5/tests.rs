#[cfg(test)]
mod tests {
    use crate::protocols::rc5::{Rc5, Rc5Command};
    use crate::recv::*;

    #[test]
    fn rc5_command() {
        let cmd = Rc5Command::new(20, 15, false);
        assert_eq!(cmd, Rc5Command::from_bits(cmd.to_bits()))
    }

    #[test]
    fn test_bufrecv() {
        let dists = [
            0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37, 0, 37,
            34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
        ];

        let r: BufferedReceiver<Rc5> = BufferedReceiver::new(&dists, 40_000);

        for c in r {
            println!("c = {:?}", c);
            assert_eq!(c.addr, 20);
            assert_eq!(c.cmd, 9);
        }
    }

    #[test]
    fn command() {
        let dists = [
            0, 37, 34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37, 0, 37,
            34, 72, 72, 73, 70, 72, 36, 37, 34, 36, 36, 36, 71, 73, 35, 37, 70, 37,
        ];

        let mut recv: EventReceiver<Rc5> = EventReceiver::new(40_000);
        let mut edge = false;
        let mut tot = 0;

        for dist in dists.iter() {
            edge = !edge;
            tot += *dist;

            let s0 = recv.sm.state;
            let cmd = recv.edge_event(edge, tot);
            let s1 = recv.sm.state;

            println!("{} ({}): {:?} -> {:?}", edge as u32, dist, s0, s1,);

            if let Ok(Some(cmd)) = cmd {
                println!("cmd: {:?}", cmd);
                assert_eq!(cmd.addr, 20);
                assert_eq!(cmd.cmd, 9);
            }
        }
    }

    #[test]
    #[rustfmt::skip]
    fn command_mixed() {
        let dists = [
            57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36, 71, 36,
            26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36, 71, 36,
            // From another rc5 like protocol but not standard rc5, should be ignored by the receiver
            10254, 37, 35, 37, 34, 37, 35, 37, 35, 73, 34, 38, 70, 37, 141, 38, 34, 37, 35, 37, 34, 38, 34, 37, 70, 73, 35, 37, 35, 37, 34, 38, 34, 37, 34, 38,
            50973, 38, 34, 73, 70, 73, 70, 74, 34, 37, 35, 37, 34, 38, 34, 37, 35, 37, 34, 38, 70, 37,
        ];

        let mut recv: EventReceiver<Rc5> = EventReceiver::new(40_000);
        let mut edge = false;
        let mut tot = 0;

        for dist in dists.iter() {
            edge = !edge;
            tot += *dist;

            let s0 = recv.sm.state;
            let cnt = recv.sm.rc5cntr;
            let cmd = recv.edge_event(edge, tot);
            let s1 = recv.sm.state;

            println!(
                "{} {} ({}): {:?} -> {:?}",
                edge as u32,
                cnt,
                dist,
                s0,
                s1,
            );

            if let Ok(Some(cmd)) = cmd {
                println!("cmd: {:?}", cmd);
                assert_eq!(cmd.addr, 20);
                assert_eq!(cmd.cmd, 1);
            }
        }

        let r: BufferedReceiver<Rc5> = BufferedReceiver::new(&dists, 40_000);

        for c in r {
            println!("c = {:?}", c);
            assert_eq!(c.addr, 20);
            assert_eq!(c.cmd, 1);
        }

    }

    /*
    #[test]
    fn rc5_transmit() {
        use crate::send::{State, Sender};
        let mut tx = Rc5Sender::new(40_000);

        tx.load(Rc5Command::new(20, 9, false));

        println!("bits: {:X?}", tx.bits);

        let mut last_enable = false;
        let mut last_ts = 0;

        for ts in 0..2000 {
            let state = tx.step(ts);

            if let State::Transmit(v) = state {
                if v != last_enable {
                    last_enable = v;
                    let delta = ts - last_ts;
                    println!("state: {}: {:?}", delta, state);
                    last_ts = ts;
                }
            }
        }
    }

     */
}
