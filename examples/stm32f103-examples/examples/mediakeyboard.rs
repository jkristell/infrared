#![no_main]
#![no_std]

use cortex_m::asm;
use embedded_hal::digital::v2::OutputPin;
use rtic::app;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Floating, Input},
    pac::TIM2,
    prelude::*,
    timer::{CountDownTimer, Event, Timer},
    usb::{Peripheral, UsbBus, UsbBusType},
};

use usb_device::{bus, prelude::*};

use usbd_hid::{
    descriptor::{generator_prelude::*, MediaKey, MediaKeyboardReport},
    hid_class::HIDClass,
};

use cortex_m::peripheral::DWT;
use infrared::{hal::PeriodicReceiver, protocols::Nec, remotes::nec::SpecialForMp3, Button};

use rtic::cyccnt::{Instant, U32Ext};

type RecvPin = PB8<Input<Floating>>;

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

#[app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_kbd: HIDClass<'static, UsbBusType>,
        timer: CountDownTimer<TIM2>,
        receiver: PeriodicReceiver<Nec, RecvPin>,
    }

    #[init]
    fn init(mut cx: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<bus::UsbBusAllocator<UsbBusType>> = None;

        rtt_init_print!();

        cx.core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        cx.core.DWT.enable_cycle_counter();

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);

        // BluePill board has a pull-up resistor on the D+ line.
        // Pull the D+ pin down to send a RESET condition to the USB bus.
        // This forced reset is needed only for development, without it host
        // will not reset your device when you upload new firmware.
        let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
        usb_dp.set_low().unwrap();
        asm::delay(clocks.sysclk().0 / 100);

        let usb_dm = gpioa.pa11;
        let usb_dp = usb_dp.into_floating_input(&mut gpioa.crh);

        let usb = Peripheral {
            usb: cx.device.USB,
            pin_dm: usb_dm,
            pin_dp: usb_dp,
        };

        *USB_BUS = Some(UsbBus::new(usb));

        let usb_kbd = HIDClass::new(USB_BUS.as_ref().unwrap(), MediaKeyboardReport::desc(), 64);

        let usb_dev = UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Infrared")
            .product("Mediakeyboard")
            .serial_number("TEST")
            .device_class(0x03) // HID
            .build();

        let SAMPLERATE = 20_000;

        let mut timer =
            Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(SAMPLERATE.hz());
        timer.listen(Event::Update);

        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);

        let receiver = PeriodicReceiver::new(pin, SAMPLERATE);

        init::LateResources {
            usb_dev,
            usb_kbd,
            timer,
            receiver,
        }
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        rprintln!("Setup done: in idle");
        loop {
            continue;
        }
    }

    #[task(binds = USB_HP_CAN_TX, priority = 3, resources = [usb_dev, usb_kbd])]
    fn usb_tx(mut cx: usb_tx::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.usb_kbd);
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, resources = [usb_dev, usb_kbd])]
    fn usb_rx0(mut cx: usb_rx0::Context) {
        usb_poll(&mut cx.resources.usb_dev, &mut cx.resources.usb_kbd);
    }

    #[task(binds = TIM2, resources = [timer, receiver, ], spawn = [keydown])]
    fn tim2_irq(cx: tim2_irq::Context) {
        let tim2_irq::Resources { timer, receiver } = cx.resources;

        timer.clear_update_interrupt_flag();

        match receiver.poll_button::<SpecialForMp3>() {
            Ok(Some(button)) => {
                rprintln!("Received: {:?}", button);

                let key = button_to_mediakey(button);

                if let Err(err) = cx.spawn.keydown(key) {
                    rprintln!("Failed to spawn keydown: {:?}", err);
                }
            }
            Ok(None) => {}
            Err(err) => {
                rprintln!("Error: {:?}", err);
            }
        }
    }

    #[task(resources = [usb_kbd], schedule = [keyup])]
    fn keydown(mut cx: keydown::Context, key: MediaKey) {
        rprintln!("keydown  @ {:?}", Instant::now());
        cx.resources.usb_kbd.lock(|kbd| send_keycode(kbd, key));

        if let Err(err) = cx.schedule.keyup(Instant::now() + 4_000_000.cycles()) {
            rprintln!("Failed to schedule keyup: {:?}", err);
        }
    }

    #[task(resources = [usb_kbd])]
    fn keyup(mut cx: keyup::Context) {
        rprintln!("keyup  @ {:?}", Instant::now());
        cx.resources
            .usb_kbd
            .lock(|kbd| send_keycode(kbd, MediaKey::Zero));
    }

    extern "C" {
        fn USART1();
    }
};

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    usb_kbd: &mut HIDClass<'static, B>,
) {
    while usb_dev.poll(&mut [usb_kbd]) {}
}

fn send_keycode(kbd: &HIDClass<UsbBusType>, key: MediaKey) {
    let report = MediaKeyboardReport {
        usage_id: key.into(),
    };

    loop {
        let r = kbd.push_input(&report);
        match r {
            Ok(_) => break,
            Err(UsbError::WouldBlock) => {
                continue;
            }
            Err(_) => break,
        }
    }
}

fn button_to_mediakey(b: Button) -> MediaKey {
    match b {
        Button::Play_Paus => MediaKey::PlayPause,
        Button::Plus => MediaKey::VolumeIncrement,
        Button::Minus => MediaKey::VolumeDecrement,
        Button::Next => MediaKey::NextTrack,
        Button::Prev => MediaKey::PrevTrack,
        Button::Stop => MediaKey::Stop,
        _ => MediaKey::Zero,
    }
}
