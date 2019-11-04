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
    hal::{self, MultiReceiver},
    nec::*,
    rc5::*,
};

const FREQ: u32 = 40_000;

static mut TIMER: Option<Timer<TIM2>> = None;
// Receiver
static mut HWRECV: Option<MultiReceiver<NecReceiver, Rc5Receiver, PB8<Input<Floating>>>> = None;
//static mut HWRECV: Option<hal::Receiver<NecReceiver, PB8<Input<Floating>>>> = None;


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
    let irinpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, FREQ.hz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    let nec = NecReceiver::new(FREQ);
    let rc5 = Rc5Receiver::new(FREQ);
    let hwr = MultiReceiver::new(nec, rc5, irinpin);
    //let hwr = hal::Receiver::new(nec, irinpin);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        HWRECV.replace(hwr);
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
    //static mut PINVAL: bool = false;

    let hwr = unsafe { HWRECV.as_mut().unwrap() };

    if let Some((neccmd, rc5cmd)) = hwr.step(*COUNT).unwrap() {
        if let Some(cmd) = neccmd {
            hprintln!("nec: {} {}", cmd.addr, cmd.cmd).unwrap();
        }

        if let Some(cmd) = rc5cmd {
            use infrared::remotes::rc5::Rc5CdPlayer;
            use infrared::remotes::RemoteControl;

            // Print the command if recognized as a Rc5 CD-player command
            if let Some(decoded) = Rc5CdPlayer.decode_with_address(cmd) {
                hprintln!("rc5 CD: {:?}", decoded).unwrap();
            } else {
                hprintln!("rc5: {} {}", cmd.addr, cmd.cmd).unwrap();
            }
        }
    }

    // Clear the interrupt
    let timer = unsafe { &mut TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    *COUNT = COUNT.wrapping_add(1);
}

