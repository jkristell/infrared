#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    pac::{self, interrupt, TIM2, TIM4},
    prelude::*,
    pwm::{PwmChannel, C4},
    timer::{CountDownTimer, Event, Tim4NoRemap, Timer},
};

use infrared::{
    protocol::Mitsubishi,
    remotecontrol::{rc5::CdPlayer, Action, RemoteControlModel},
    sender::Sender,
};

type PwmPin = PwmChannel<TIM4, C4>;
const TIMER_FREQ: usize = 20_000;

// Global timer
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
// Transmitter
static mut TRANSMITTER: Option<Sender<PwmPin, 20_000, 1024>> = None;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let _cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let mut rcc = d.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut timer =
        Timer::tim2(d.TIM2, &clocks, &mut rcc.apb1).start_count_down((TIMER_FREQ as u32).hz());

    timer.listen(Event::Update);

    // PWM
    let mut afio = d.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = d.GPIOB.split(&mut rcc.apb2);
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let pwm = Timer::tim4(d.TIM4, &clocks, &mut rcc.apb1).pwm::<Tim4NoRemap, _, _, _>(
        irled,
        &mut afio.mapr,
        38.khz(),
    );

    let mut irpin = pwm.split();

    irpin.set_duty(irpin.get_max_duty() / 3);
    irpin.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        TRANSMITTER.replace(Sender::new(irpin));
    }

    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::TIM2);
    }

    rprintln!("Init done");
    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNTER: usize = 0;

    *COUNTER = COUNTER.wrapping_add(1);

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    let transmitter = unsafe { TRANSMITTER.as_mut().unwrap() };
    transmitter.tick();

    if *COUNTER == TIMER_FREQ * 2 {
        rprintln!("Pressing button");

        //let cmd = CdPlayer::encode(&Action::Next).unwrap();
        let cmd = infrared::protocol::MitsubishiCommand::new(false);
        transmitter.load::<Mitsubishi>(&cmd);

        *COUNTER = 0;
    }
}
