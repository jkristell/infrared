#![no_std]
#![no_main]

use bluepill_examples as _;
use defmt::info;

use cortex_m_rt::entry;

use stm32f1xx_hal::{
    pac::{self, interrupt, TIM2, TIM4},
    prelude::*,
    timer::{CounterHz, Event, PwmChannel, Tim4NoRemap, Timer, C4},
};

use infrared::{
    protocol::Rc5,
    remotecontrol::{rc5::CdPlayer, Action, RemoteControlModel},
    sender::Sender,
};

type PwmPin = PwmChannel<TIM4, C4>;
const TIMER_FREQ: u32 = 20_000;

// Global timer
static mut TIMER: Option<CounterHz<TIM2>> = None;
// Transmitter
static mut TRANSMITTER: Option<Sender<PwmPin, 20_000, 128>> = None;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let rcc = d.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut timer = Timer::new(d.TIM2, &clocks).counter_hz();

    timer.start(TIMER_FREQ.Hz()).unwrap();

    timer.listen(Event::Update);

    // PWM
    let mut afio = d.AFIO.constrain();
    let mut gpiob = d.GPIOB.split();
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let pwm =
        Timer::new(d.TIM4, &clocks).pwm_hz::<Tim4NoRemap, _, _>(irled, &mut afio.mapr, 38.kHz());

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

    info!("Init done");
    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNTER: u32 = 0;

    *COUNTER = COUNTER.wrapping_add(1);

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_interrupt(Event::Update);

    let transmitter = unsafe { TRANSMITTER.as_mut().unwrap() };
    transmitter.tick();

    if *COUNTER == TIMER_FREQ * 2 {
        info!("Pressing button");

        let cmd = CdPlayer::encode(&Action::Next).unwrap();
        transmitter.load::<Rc5>(&cmd);

        *COUNTER = 0;
    }
}
