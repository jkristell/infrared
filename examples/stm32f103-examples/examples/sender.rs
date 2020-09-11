#![no_std]
#![no_main]
#![allow(deprecated)]

use cortex_m::asm;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    pac,
    pac::{interrupt, TIM2, TIM4},
    prelude::*,
    pwm::{PwmChannel, C4},
    timer::{CountDownTimer, Event, Tim4NoRemap, Timer},
};

use infrared::{
    protocols::rc5::*,
    remotes::rc5::Rc5CdPlayer,
    sender::{PwmPinSender, Sender},
    Button, RemoteControl,
};

const TIMER_FREQ: u32 = 20_000;

// Global timer
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
// transmitter
static mut TRANSMITTER: Option<Rc5Sender> = None;
// Pwm channel
static mut PWMCHANNEL: Option<PwmChannel<TIM4, C4>> = None;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    rprintln!("{}", info);
    exit()
}

fn exit() -> ! {
    loop {
        asm::bkpt() // halt = exit probe-run
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

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

    let mut timer =
        Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1).start_count_down(TIMER_FREQ.hz());

    timer.listen(Event::Update);

    // PWM
    let mut afio = device.AFIO.constrain(&mut rcc.apb2);
    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let irled = gpiob.pb9.into_alternate_push_pull(&mut gpiob.crh);

    let pwm = Timer::tim4(device.TIM4, &clocks, &mut rcc.apb1).pwm::<Tim4NoRemap, _, _, _>(
        irled,
        &mut afio.mapr,
        38.khz(),
    );

    let mut irpin = pwm.split();

    irpin.set_duty(irpin.get_max_duty() / 2);
    irpin.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        TRANSMITTER.replace(Rc5Sender::new(TIMER_FREQ));
        PWMCHANNEL.replace(irpin);
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
    let channel = unsafe { PWMCHANNEL.as_mut().unwrap() };

    if *COUNT % TIMER_FREQ == 0 {
        let button = Button::Next;

        // The encoded result
        let cmd = Rc5CdPlayer::encode(button).unwrap();

        // You could also construct the command manually
        // let cmd = Rc5Command::new(20, 15, false);

        transmitter.load(cmd);
    }

    transmitter.step_pwm(*COUNT, channel);

    *COUNT = COUNT.wrapping_add(1);
}
