#![no_std]
#![no_main]

use bluepill_examples as _;
use cortex_m_rt::entry;
use defmt::info;
use infrared::{
    protocol::Rc5,
    remotecontrol::{rc5::CdPlayer, Action, RemoteControl},
    sender::PwmSender,
};
use stm32f1xx_hal::{
    pac::{self, interrupt, TIM2, TIM4},
    prelude::*,
    rcc,
    timer::{CounterHz, Event, PwmChannel, Tim4NoRemap, C4},
};

// The IR led, connected to PB9 (TIM4 channel 4)
type IrPwm = PwmChannel<TIM4, C4>;

// Tick rate of the sender
const TIMER_FREQ: u32 = 20_000;

// Global timer
static mut TIMER: Option<CounterHz<TIM2>> = None;
// Transmitter
static mut TRANSMITTER: Option<PwmSender<IrPwm>> = None;

#[entry]
fn main() -> ! {
    let _cp = cortex_m::Peripherals::take().unwrap();
    let d = pac::Peripherals::take().unwrap();

    let mut flash = d.FLASH.constrain();
    let mut rcc = d.RCC.freeze(
        rcc::Config::hse(8.MHz()).sysclk(48.MHz()).pclk1(24.MHz()),
        &mut flash.acr,
    );

    let mut timer = d.TIM2.counter_hz(&mut rcc);

    timer.start(TIMER_FREQ.Hz()).unwrap();

    timer.listen(Event::Update);

    // 38 kHz carrier on PB9
    let mut afio = d.AFIO.constrain(&mut rcc);
    let mut gpiob = d.GPIOB.split(&mut rcc);
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let pwm = d
        .TIM4
        .pwm_hz::<Tim4NoRemap, _, _>(irled, &mut afio.mapr, 38.kHz(), &mut rcc);

    let mut irpwm = pwm.split();

    // Keep the channel enabled. The PwmSender turns the carrier on and off
    // by switching the duty cycle between zero and a third
    irpwm.enable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        TRANSMITTER.replace(PwmSender::with_duty_fraction(irpwm, TIMER_FREQ, 1, 3));
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
    transmitter.tick().unwrap();

    if *COUNTER == TIMER_FREQ * 2 {
        info!("Pressing button");

        let cmd = CdPlayer::encode(&Action::Next).unwrap();
        if transmitter.load::<Rc5>(&cmd).is_err() {
            info!("Sender busy");
        }

        *COUNTER = 0;
    }
}
