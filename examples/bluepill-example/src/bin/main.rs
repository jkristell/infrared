#![no_std]
#![no_main]
#![allow(unused)]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use cortex_m::iprintln;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2},
    timer::{Event, Timer},
};

use heapless::consts::*;
use heapless::spsc::Queue;

use infrared::{
    nec::{NecCmd, NecReceiver, NecResult},
    Receiver,
};

const FREQ: u32 = 20_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: NecReceiver = NecReceiver::new(FREQ);
static mut CQ: Option<Queue<NecResult, U8>> = None;

#[entry]
fn main() -> ! {
    let mut core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let mut rcc = device.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(16.mhz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split(&mut rcc.apb2);
    let mut irpin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::tim2(device.TIM2, 20.khz(), clocks, &mut rcc.apb1);
    timer.listen(Event::Update);

    unsafe {
        TIMER.replace(timer);
        IRPIN.replace(irpin);
    }

    // Enable the external interrupt
    core.NVIC.enable(pac::Interrupt::TIM2);

    unsafe { CQ = Some(Queue::new()) };
    let mut consumer = unsafe { CQ.as_mut().unwrap().split().1 };
    let stim = &mut core.ITM.stim[0];

    iprintln!(stim, "Testar");

    loop {
        if let Some(result) = consumer.dequeue() {
            match result {
                Ok(Some(NecCmd::Command(button))) => {
                    if !button.verify() {
                        hprintln!("Failed to verify cmd {}", button.command()).unwrap();
                    }

                    if let Some(name) = command_to_str(button.command()) {
                        hprintln!("cmd: {}", name).unwrap();
                    } else {
                        hprintln!("unknown: {}", button.command()).unwrap();
                    }
                }
                Ok(Some(NecCmd::Repeat)) => hprintln!("repeat").unwrap(),
                Ok(None) => (),
                Err(err) => hprintln!("Err: {:?}", err).unwrap(),
            }
        }
    }
}

#[interrupt]
fn TIM2() {
    static mut COUNT: u32 = 0;
    static mut PINVAL: bool = false;

    // Clear the interrupt
    let timer = unsafe { &mut TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();

    // Read the value of the pin (active low)
    let new_pinval = unsafe { IRPIN.as_ref().unwrap().is_low() };

    // Only look for positive edges
    let pos_edge = *PINVAL == false && new_pinval == true;

    if pos_edge {
        let nec = unsafe { &mut NEC };
        let result = nec.event(*COUNT);

        let mut producer = unsafe { CQ.as_mut().unwrap().split().0 };
        producer.enqueue(result).ok().unwrap();

        if result.is_err() {
            nec.reset();
        }
    }

    *PINVAL = new_pinval;
    *COUNT += 1;
}

// Mappings for "Special for MP3" Remote
fn command_to_str(cmd: u8) -> Option<&'static str> {
    match cmd {
        69 => Some("Power"),
        70 => Some("Mode"),
        71 => Some("Mute"),
        68 => Some("Play/Paus"),
        64 => Some("Prev"),
        67 => Some("Next"),
        7 => Some("Eq"),
        21 => Some("Minus"),
        9 => Some("Plus"),
        22 => Some("0"),
        25 => Some("Shuffle"),
        13 => Some("U/SD"),
        12 => Some("1"),
        24 => Some("2"),
        94 => Some("3"),
        8 => Some("4"),
        28 => Some("5"),
        90 => Some("6"),
        66 => Some("7"),
        82 => Some("8"),
        74 => Some("9"),
        _ => None,
    }
}
