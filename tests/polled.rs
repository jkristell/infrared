#[cfg(feature = "rc5")]
#[test]
fn polled_rc5() {
    use infrared::{
        protocol::{Rc5, Rc5Command},
        PeriodicPoll, Receiver,
    };

    // Rc5 cmd data sampled at 40 kHz
    let data = [
        57910, 36, 36, 36, 35, 37, 35, 72, 71, 72, 36, 36, 36, 36, 35, 36, 36, 36, 35, 36, 36, 36,
        71, 36, 26605, 36, 36, 71, 72, 72, 71, 72, 36, 36, 35, 36, 36, 36, 35, 37, 35, 36, 36, 36,
        71, 36,
    ];

    let mut recv: PeriodicPoll<Rc5> = Receiver::builder().rc5().frequency(40_000).build_polled();

    let mut pinstate = false;

    let mut res = None;

    for n in data {
        for _p in 0..n {
            if let Ok(Some(cmd)) = recv.poll(pinstate) {
                res = Some(cmd);
            }
        }
        pinstate = !pinstate;
    }

    assert_eq!(
        res,
        Some(Rc5Command {
            addr: 20,
            cmd: 1,
            start: 3,
            toggle: false
        })
    );
}
