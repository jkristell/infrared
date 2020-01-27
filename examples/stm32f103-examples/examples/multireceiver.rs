#![no_std]
#![no_main]
#![allow(deprecated)]

use cortex_m::asm;
use cortex_m_rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{CountDownTimer, Event, Timer},
};

use infrared::{
    hal::PeriodicReceiver5,
    protocols::{Nec, NecSamsung, Rc5, Rc6, Sbp},
    remotes::rc5::Rc5CdPlayer,
    RemoteControl,
};

type RecvPin = PB8<Input<Floating>>;

const SAMPLERATE: u32 = 20_000;
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
static mut RECEIVER: Option<PeriodicReceiver5<Nec, NecSamsung, Rc5, Rc6, Sbp, RecvPin>> = None;

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

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let inpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer =
        Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1).start_count_down(SAMPLERATE.hz());

    timer.listen(Event::Update);

    // Create a receiver that reacts on 3 different kinds of remote controls
    let receiver = PeriodicReceiver5::new(inpin, SAMPLERATE);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    // Enable the timer interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    rprintln!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok((neccmd, nescmd, rc5cmd, rc6cmd, sbpcmd)) = receiver.poll() {
        // We have a NEC Command
        if let Some(cmd) = neccmd {
            rprintln!("nec: {} {}", cmd.addr, cmd.cmd);
        }

        // We have Samsung-flavoured NEC Command
        if let Some(cmd) = nescmd {
            rprintln!("nec: {} {}", cmd.addr, cmd.cmd);
        }

        // We have a Rc5 Command
        if let Some(cmd) = rc5cmd {
            // Print the command if recognized as a Rc5 CD-player command
            if let Some(decoded) = Rc5CdPlayer::decode(cmd) {
                rprintln!("rc5(CD): {:?}", decoded);
            } else {
                rprintln!("rc5: {} {}", cmd.addr, cmd.cmd);
            }
        }

        if let Some(cmd) = rc6cmd {
            rprintln!("rc6: {} {}", cmd.addr, cmd.cmd);
        }

        if let Some(cmd) = sbpcmd {
            rprintln!("sbp: {} {}", cmd.address, cmd.command);
        }
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();
}
