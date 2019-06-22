#![no_main]
#![no_std]
#![allow(deprecated)]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use nucleo_f401re::{
    gpio::{gpiob::PB8, Floating, Input},
    peripheral::Peripherals,
    prelude::*,
    stm32,
    stm32::interrupt,
    timer::{Event, Timer},
    Interrupt,
};
use panic_semihosting as _;
use infrared::{
    nec::{NecResult, NecCmd, NecReceiver},
    Receiver,
};

use heapless::consts::*;
use heapless::spsc::Queue;

// 50 us = 20_000 Hz
const FREQ: u32 = 20_000;

// Global data
static mut IRPIN: Option<PB8<Input<Floating>>> = None;
static mut NEC: NecReceiver = NecReceiver::new(FREQ);
static mut CQ: Option<Queue<NecResult, U8>> = None;

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

#[entry]
fn main() -> ! {
    let device = stm32::Peripherals::take().unwrap();
    let mut core = Peripherals::take().unwrap();

    device.RCC.apb2enr.modify(|_, w| w.syscfgen().enabled());

    let rcc = device.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(84.mhz()).freeze();

    let gpiob = device.GPIOB.split();
    let irpin = gpiob.pb8.into_floating_input();

    unsafe {
        IRPIN.replace(irpin);
    }

    // Setup the timer for 50us operation
    let mut timer2 = Timer::tim2(device.TIM2, FREQ.hz(), clocks);
    timer2.listen(Event::TimeOut);

    // Enable the external interrupt
    core.NVIC.enable(Interrupt::TIM2);

    unsafe { CQ = Some(Queue::new()) };
    let mut consumer = unsafe { CQ.as_mut().unwrap().split().1 };

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
    unsafe { (*stm32::TIM2::ptr()).sr.modify(|_, w| w.uif().clear_bit()); }

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
