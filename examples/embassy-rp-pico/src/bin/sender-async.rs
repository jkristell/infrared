//! Async NEC sender on a Raspberry Pi Pico using embassy.
//!
//! Connect an IR led (with a suitable driver transistor and resistor) to GPIO14.

#![no_std]
#![no_main]

use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::pwm::{Config, Pwm};
use embassy_time::Timer;
use infrared::{
    protocol::{Nec, nec::NecCommand},
    sender::AsyncSender,
};
use panic_probe as _;

/// Carrier frequency in Hz
const CARRIER: u32 = 38_000;

#[embassy_executor::main(
    executor = "embassy_rp::executor::Executor",
    entry = "cortex_m_rt::entry"
)]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // 38 kHz carrier on GPIO14 (PWM slice 7, channel A)
    let mut config = Config::default();
    config.top = (embassy_rp::clocks::clk_sys_freq() / CARRIER - 1) as u16;
    let pwm = Pwm::new_output_a(p.PWM_SLICE7, p.PIN_14, config);

    let mut sender = AsyncSender::embassy(pwm);

    let cmd = NecCommand {
        addr: 0,
        cmd: 12,
        repeat: false,
    };

    loop {
        info!("Sending {:?}", cmd);
        if sender.send::<Nec>(&cmd).await.is_err() {
            warn!("Pwm error");
        }
        Timer::after_secs(1).await;
    }
}
