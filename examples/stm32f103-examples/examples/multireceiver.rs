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
    timer::{Event, Timer, CountDownTimer},
};

use infrared::{
    InfraredReceiver5,
    nec::*,
    rc5::*,
    rc6::*,
    sbp::*,
    remotes::{
        RemoteControl,
        rc5::Rc5CdPlayer
    },
};


const TIMER_FREQ: u32 = 40_000;

static mut TIMER: Option<CountDownTimer<TIM2>> = None;

// Receiver for multiple protocols
static mut RECEIVER: Option<InfraredReceiver5<PB8<Input<Floating>>,
    Nec,
    NecSamsung,
    Rc5,
    Rc6,
    Sbp,
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

    let mut timer = Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1)
        .start_count_down(TIMER_FREQ.hz());

    timer.listen(Event::Update);

    // Create a receiver that reacts on 3 different kinds of remote controls
    let receiver = InfraredReceiver5::new(inpin, TIMER_FREQ);

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
    static mut SAMPLECOUNTER: u32 = 0;

    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok((neccmd, nescmd, rc5cmd, rc6cmd, sbpcmd)) = receiver.step(*SAMPLECOUNTER) {

        // We have a NEC Command
        if let Some(cmd) = neccmd {
            hprintln!("nec: {} {}", cmd.addr, cmd.cmd).unwrap();
        }

        // We have Samsung-flavoured NEC Command
        if let Some(cmd) = nescmd {
            hprintln!("nec: {} {}", cmd.addr, cmd.cmd).unwrap();
        }

        // We have a Rc5 Command
        if let Some(cmd) = rc5cmd {
            // Print the command if recognized as a Rc5 CD-player command
            if let Some(decoded) = Rc5CdPlayer::decode(cmd) {
                hprintln!("rc5(CD): {:?}", decoded).unwrap();
            } else {
                hprintln!("rc5: {} {}", cmd.addr, cmd.cmd).unwrap();
            }
        }

        if let Some(cmd) = rc6cmd {
            hprintln!("rc6: {} {}", cmd.addr, cmd.cmd).ok();
        }

        if let Some(cmd) = sbpcmd {
            hprintln!("sbp: {} {}", cmd.address, cmd.command).ok();
        }

    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    *SAMPLECOUNTER = SAMPLECOUNTER.wrapping_add(1);
}

