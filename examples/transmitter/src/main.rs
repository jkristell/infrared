#![no_std]
#![no_main]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB9, Alternate, PushPull},
    pac,
    prelude::*,
    pwm::{Pins, Pwm, C4},
    stm32::{interrupt, TIM2, TIM4},
    timer::{Event, Timer},
};

use infrared::{
    prelude::*,
    prelude::hal::*,
    rc5::*,
};

const FREQ: u32 = 20_000;

// Global timer
static mut TIMER: Option<Timer<TIM2>> = None;
// transmitter
static mut NECTX: Option<Rc5Transmitter> = None;
// Pwm channel
static mut PWM: Option<Pwm<TIM4, C4>> = None;
// Our remote control we want to act like

struct PwmChannels(PB9<Alternate<PushPull>>);
impl Pins<TIM4> for PwmChannels {
    const REMAP: u8 = 0b00;
    const C1: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = true; // PB9
    type Channels = Pwm<TIM4, C4>;
}

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

    assert!(clocks.usbclk_valid());

    let mut timer = Timer::tim2(device.TIM2, FREQ.hz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    // PWM
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let mut c4: Pwm<TIM4, C4> = device.TIM4.pwm(
        PwmChannels(irled),
        &mut afio.mapr,
        38.khz(),
        clocks,
        &mut rcc.apb1,
    );
    // Set the duty cycle of channel 0 to 50%
    c4.set_duty(c4.get_max_duty() / 2);
    c4.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        NECTX.replace(Rc5Transmitter::new(FREQ));
        PWM.replace(c4);
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
    let timer = unsafe { &mut TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    // Get handles to the transmitter and the pwm
    let transmitter = unsafe { NECTX.as_mut().unwrap() };
    let pwm = unsafe { PWM.as_mut().unwrap() };

    if *COUNT % FREQ == 0 {
        // Next Channel on my Samsung TV
        let cmd = Rc5Command::new(20, 15, false);
        transmitter.load(cmd);
    }

    transmitter.pwmstep(*COUNT, pwm);

    *COUNT = COUNT.wrapping_add(1);
}
