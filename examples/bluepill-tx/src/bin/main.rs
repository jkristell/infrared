#![no_std]
#![no_main]
#![allow(deprecated)]

use panic_semihosting as _;

use cortex_m::asm::delay;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32f1xx_hal::{
    gpio::{gpiob::PB9, PushPull, Alternate},
    pwm::{Pins, Pwm, C4},
    pac,
    prelude::*,
    stm32::{interrupt, TIM2, TIM4},
    timer::{Event, Timer},
};
use stm32_usbd::UsbBus;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use heapless::consts::*;
use heapless::spsc::Queue;

use infrared::{
    nec::{NecType, NecTransmitter},
    nec::remotes::*,
    Transmitter, TransmitterState,
    RemoteControl,
};

const FREQ: u32 = 20_000;

static mut TIMER: Option<Timer<TIM2>> = None;
static mut NECTX: Option<NecTransmitter> = None;
static mut PWM: Option<Pwm<TIM4, C4>> = None;

static mut TXQ: Option<Queue<SamsungTvAction, U8>> = None;

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

    let mut gpioa = device.GPIOA.split(&mut rcc.apb2);

    // BluePill board has a pull-up resistor on the D+ line.
    // Pull the D+ pin down to send a RESET condition to the USB bus.
    let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
    usb_dp.set_low();
    delay(clocks.sysclk().0 / 100);

    let usb_dm = gpioa.pa11;
    let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

    let usb_bus = UsbBus::new(device.USB, (usb_dm, usb_dp));

    let mut serial = SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x5824, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(USB_CLASS_CDC)
        .build();


    let mut timer = Timer::tim2(device.TIM2, 20.khz(), clocks, &mut rcc.apb1);
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
        &mut rcc.apb1
    );
    // Set the duty cycle of channel 0 to 50%
    c4.set_duty(c4.get_max_duty() / 2);
    c4.disable();

    // Safe because the devices are only used in the interrupt handler
    unsafe {
        TIMER.replace(timer);
        let per: u32 = (1 * 1000) / (FREQ / 1000);
        NECTX.replace(NecTransmitter::new(NecType::Samsung, per));
        PWM.replace(c4);
    }

    core.NVIC.enable(pac::Interrupt::TIM2);

    // Initialize the queue
    unsafe {
        TXQ = Some(Queue::new());
    };

    let mut txq = unsafe { TXQ.as_mut().unwrap().split().0 };

    loop {
        // Handle USB
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {

                    // Echo back in upper case
                    for c in buf[0..count].iter_mut() {
                        // Enqueue all commands

                        let action = match *c as char {
                            'o' => SamsungTvAction::Power,
                            'n' => SamsungTvAction::ChannelListNext,
                            'p' => SamsungTvAction::ChannelListPrev,
                            _ => SamsungTvAction::Teletext,
                        };

                        txq.enqueue(action).ok().unwrap();

                        if 0x61 <= *c && *c <= 0x7a {
                            *c = *c + 1;
                        }
                    }

                    let mut write_offset = 0;
                    while write_offset < count {
                        match serial.write(&buf[write_offset..count]) {
                            Ok(len) if len > 0 => {
                                write_offset += len;
                            },
                            _ => {},
                        }
                    }
                }
                _ => {}
            }
        }
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

    // Update the state in the transmitter
    let state = transmitter.step(*COUNT);

    // Get a handle to our action queue
    let mut action_queue = unsafe { TXQ.as_mut().unwrap().split().1 };

    match state {
        TransmitterState::Idle => {
            // Make sure the Pwm is disabled
            pwm.disable();
            // Check queue for new commands
            if let Some(action) = action_queue.dequeue() {
                // The the remote for the given protocol (variant) we want to act as
                let remote = SamsungTv;
                // Initialize the transfer
                transmitter.init(remote.encode(action));
            }
        },
        // The state machine wants us to activate the pwm
        TransmitterState::Transmit(true) => pwm.enable(),
        // And disable it
        TransmitterState::Transmit(false) => pwm.disable(),
        // Error when sendinf
        TransmitterState::Err => hprintln!("Err!!").unwrap(),
    }

    *COUNT += 1;
}


