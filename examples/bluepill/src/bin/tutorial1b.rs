#![no_std]
#![no_main]

use bluepill_examples as _;
use defmt::{Debug2Format, info};

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac::{self, interrupt, TIM2},
    prelude::*,
    timer::{CounterHz, Event, Timer},
};

use infrared::{
    protocol::{Rc6, Rc6Command},
    receiver::{PinInput, Poll},
    remotecontrol::{Action, Button, DeviceType, RemoteControlModel},
    ProtocolId, Receiver,
};

// Sample rate
const TIMER_FREQ: u32 = 100_000;

// Our receivertype
type IrReceiver = Receiver<Rc6, Poll, PinInput<PB8<Input<Floating>>>, Button<Rc6Tv>>;

// Globals
static mut TIMER: Option<CounterHz<TIM2>> = None;
static mut RECEIVER: Option<IrReceiver> = None;

#[derive(Debug, Default)]
struct Rc6Tv;

impl RemoteControlModel for Rc6Tv {
    const MODEL: &'static str = "Rc6 Tv";
    const DEVTYPE: DeviceType = DeviceType::TV;
    const PROTOCOL: ProtocolId = ProtocolId::Rc6;
    const ADDRESS: u32 = 0;
    type Cmd = Rc6Command;
    const BUTTONS: &'static [(u32, Action)] = &[
        // Cmdid to Button mappings
        (1, Action::One),
        (2, Action::Two),
        (3, Action::Three),
        (4, Action::Four),
        (5, Action::Five),
        (6, Action::Six),
        (7, Action::Seven),
        (8, Action::Eight),
        (9, Action::Nine),
        (12, Action::Power),
        (76, Action::VolumeUp),
        (77, Action::VolumeDown),
        (60, Action::Teletext),
    ];
}

#[entry]
fn main() -> ! {
    let _core = cortex_m::Peripherals::take().unwrap();
    let device = pac::Peripherals::take().unwrap();

    let mut flash = device.FLASH.constrain();
    let rcc = device.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .pclk1(24.MHz())
        .freeze(&mut flash.acr);

    let mut gpiob = device.GPIOB.split();
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer = Timer::new(device.TIM2, &clocks).counter_hz();
    timer.start(TIMER_FREQ.Hz()).unwrap();
    timer.listen(Event::Update);

    let receiver = Receiver::with_pin(TIMER_FREQ, pin);

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        RECEIVER.replace(receiver);
    }

    unsafe {
        // Enable the timer interrupt
        pac::NVIC::unmask(pac::Interrupt::TIM2);
    }

    info!("Ready!");

    loop {
        continue;
    }
}

#[interrupt]
fn TIM2() {
    let receiver = unsafe { RECEIVER.as_mut().unwrap() };

    if let Ok(Some(cmd)) = receiver.poll() {
        use Action::*;

        match cmd.action() {
            Some(Teletext) => info!("Teletext!"),
            Some(Power) => info!("Power on/off"),
            _ => info!("cmd: {:?}", Debug2Format(&cmd)),
        };
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_interrupt(Event::Update);
}
