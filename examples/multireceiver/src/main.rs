#![no_std]
#![no_main]
#![allow(deprecated)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32f1xx_hal::{
    gpio::{
        gpiob::PB8,
        Floating, Input
    },
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{Event, Timer},
};

use infrared::{
    hal::Receiver3,
    nec::*,
    rc5::*,
    remotes::{
        RemoteControl,
        rc5::Rc5CdPlayer
    },
};


const TIMER_FREQ: u32 = 40_000;

static mut TIMER: Option<Timer<TIM2>> = None;

// Receiver for multiple protocols
static mut RECEIVER: Option<Receiver3<PB8<Input<Floating>>,
    NecReceiver,
    NecSamsungReceiver,
    Rc5Receiver,
>> = None;


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

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let inpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, TIMER_FREQ.hz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    let nec = NecReceiver::new(TIMER_FREQ);
    let nes = NecSamsungReceiver::new(TIMER_FREQ);
    let rc5 = Rc5Receiver::new(TIMER_FREQ);
    let receiver = Receiver3::new(inpin,
                                  nec,
                                  nes,
                                  rc5);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    // Enable the timer interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    hprintln!("Ready!").unwrap();

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNT: u32 = 0;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok((neccmd, nescmd, rc5cmd)) = receiver.step(*COUNT) {

        if let Some(cmd) = neccmd {
            hprintln!("nec: {} {}", cmd.addr, cmd.cmd).unwrap();
        }

        if let Some(cmd) = nescmd {
            hprintln!("nec: {} {}", cmd.addr, cmd.cmd).unwrap();
        }

        if let Some(cmd) = rc5cmd {
            // Print the command if recognized as a Rc5 CD-player command
            if let Some(decoded) = Rc5CdPlayer.decode_with_address(cmd) {
                hprintln!("rc5(CD): {:?}", decoded).unwrap();
            } else {
                hprintln!("rc5: {} {}", cmd.addr, cmd.cmd).unwrap();
            }
        }
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    *COUNT = COUNT.wrapping_add(1);
}

