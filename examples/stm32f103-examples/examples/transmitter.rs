#![no_std]
#![no_main]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    pwm::{Pwm, C4},
    stm32::{interrupt, TIM2, TIM4},
    timer::{Event, Timer, CountDownTimer},
};

use infrared::{
    Transmitter,
    PwmTransmitter,
    rc5::*,
};

use infrared::remotes::{
    *,
    rc5::Rc5CdPlayer,
};

const TIMER_FREQ: u32 = 20_000;

// Global timer
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
// transmitter
static mut TRANSMITTER: Option<Rc5Transmitter> = None;
// Pwm channel
static mut PWM: Option<Pwm<TIM4, C4>> = None;


#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .freeze(&mut flash.acr);

    let mut timer = Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1)
        .start_count_down(TIMER_FREQ.hz());

    timer.listen(Event::Update);

    // PWM
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let mut pwm = Timer::tim4(device.TIM4, &clocks, &mut rcc.apb1)
        .pwm(irled, &mut afio.mapr, 38.khz());

    pwm.set_duty(pwm.get_max_duty() / 2);
    pwm.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        TRANSMITTER.replace(Rc5Transmitter::new(TIMER_FREQ));
        PWM.replace(pwm);
    }

    core.NVIC.enable(pac::Interrupt::TIM2);

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNT: u32 = 0;

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    // Get handles to the transmitter and the pwm
    let transmitter = unsafe { TRANSMITTER.as_mut().unwrap() };
    let pwm = unsafe { PWM.as_mut().unwrap() };

    if *COUNT % TIMER_FREQ == 0 {

        let button = StandardButton::Next;

        // The encoded result
        let cmd = Rc5CdPlayer::encode(button).unwrap();

        // You could also construct the command manually
        // let cmd = Rc5Command::new(20, 15, false);

        transmitter.load(cmd);
    }

    transmitter.pwmstep(*COUNT, pwm);

    *COUNT = COUNT.wrapping_add(1);
}
