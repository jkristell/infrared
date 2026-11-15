//! Async NEC receiver on a Raspberry Pi Pico using embassy.
//!
//! Connect the output of an IR receiver module (e.g. a TSOP382) to GPIO12.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
use panic_probe as _;

#[embassy_executor::main(executor = "embassy_rp::executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let pin = Input::new(p.PIN_12, Pull::None);

    let mut receiver = infrared::receiver().nec().build_embassy(pin);

    info!("Waiting for IR commands");

    loop {
        match receiver.receive().await {
            Ok(cmd) => info!("{:?}", cmd),
            Err(_) => warn!("Pin error"),
        }
    }
}
