#![no_std]
#![no_main]

use cortex_m_rt::entry;
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac::{self, interrupt, TIM2},
    prelude::*,
    timer::{CountDownTimer, Event, Timer},
};

use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use infrared::{
    protocol::{Rc6, Rc6Command},
    receiver::{PinInput, Poll},
    remotecontrol::{Action, Button, DeviceType, RemoteControlModel},
    ProtocolId, Receiver,
};

// Sample rate
const TIMER_FREQ: u32 = 20_000;

// Our receivertype
type IrReceiver = Receiver<Rc6, Poll, PinInput<PB8<Input<Floating>>>, Button<Rc6Tv>>;

// Globals
static mut TIMER: Option<CountDownTimer<TIM2>> = None;
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
    rtt_init_print!();
    let _core = cortex_m::Peripherals::take().unwrap();
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
    let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

    let mut timer =
        Timer::tim2(device.TIM2, &clocks, &mut rcc.apb1).start_count_down(TIMER_FREQ.hz());

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

    rprintln!("Ready!");

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
            Some(Teletext) => rprintln!("Teletext!"),
            Some(Power) => rprintln!("Power on/off"),
            _ => rprintln!("cmd: {:?}", cmd),
        };
    }

    // Clear the interrupt
    let timer = unsafe { TIMER.as_mut().unwrap() };
    timer.clear_update_interrupt_flag();
}
