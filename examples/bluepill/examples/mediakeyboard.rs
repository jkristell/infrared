#![no_main]
#![no_std]

use cortex_m::asm;
use dwt_systick_monotonic::DwtSystick;
use embedded_hal::digital::v2::OutputPin;
use infrared::{
    protocol::NecApple,
    receiver::{Event, PinInput},
    remotecontrol::{nec::Apple2009, Action, Button},
    Receiver,
};
use panic_rtt_target as _;
use rtic::time::{duration::Milliseconds, Instant};
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal::{
    gpio::{gpiob::PB8, Edge, ExtiPin, Floating, Input},
    prelude::*,
    usb::{Peripheral, UsbBus, UsbBusType},
};
use usb_device::{bus, prelude::*};
use usbd_hid::{
    descriptor::{generator_prelude::*, MediaKey, MediaKeyboardReport},
    hid_class::HIDClass,
};

#[rtic::app(device = stm32f1xx_hal::stm32, peripherals = true, dispatchers = [USART1])]
mod app {
    use super::*;

    /// The pin connected to the infrared receiver module
    type RxPin = PB8<Input<Floating>>;

    #[monotonic(binds = SysTick, default = true)]
    type InfraMono = DwtSystick<48_000_000>;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_kbd: HIDClass<'static, UsbBusType>,
    }

    #[local]
    struct Local {
        receiver: Receiver<NecApple, Event, PinInput<crate::app::RxPin>, Button<Apple2009>>,
    }

    #[init(
        local = [usb_bus: Option<bus::UsbBusAllocator<UsbBusType>> = None],
    )]
    fn init(mut cx: init::Context) -> (Shared, Local, init::Monotonics) {
        rtt_init_print!();

        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(48.mhz())
            .pclk1(24.mhz())
            .freeze(&mut flash.acr);

        assert!(clocks.usbclk_valid());

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);

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

        let usb_bus = cx.local.usb_bus;
        usb_bus.replace(UsbBus::new(usb));

        let usb_kbd = HIDClass::new(usb_bus.as_ref().unwrap(), MediaKeyboardReport::desc(), 64);

        let usb_dev = UsbDeviceBuilder::new(usb_bus.as_ref().unwrap(), UsbVidPid(0x16c0, 0x27dd))
            .manufacturer("Infrared")
            .product("Mediakeyboard")
            .serial_number("TEST")
            .device_class(0x03) // HID
            .build();

        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut pin = gpiob.pb8.into_floating_input(&mut gpiob.crh);
        pin.make_interrupt_source(&mut afio);
        pin.trigger_on_edge(&cx.device.EXTI, Edge::RISING_FALLING);
        pin.enable_interrupt(&cx.device.EXTI);

        let mono_clock = clocks.hclk().0;
        rprintln!("Mono clock: {}", mono_clock);

        let resolution = 48_000_000;
        let receiver = Receiver::with_pin(resolution, pin);

        let monot = DwtSystick::new(&mut cx.core.DCB, cx.core.DWT, cx.core.SYST, mono_clock);

        (
            Shared { usb_dev, usb_kbd },
            Local { receiver },
            init::Monotonics(monot),
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        rprintln!("Setup done. In idle");
        loop {
            continue;
        }
    }

    #[task(binds = USB_LP_CAN_RX0, priority = 3, shared = [usb_dev, usb_kbd])]
    fn usb_rx0(cx: usb_rx0::Context) {
        let usb_dev = cx.shared.usb_dev;
        let usb_kbd = cx.shared.usb_kbd;

        (usb_dev, usb_kbd).lock(|usb_dev, usb_kbd| {
            super::usb_poll(usb_dev, usb_kbd);
        });
    }

    #[task(binds = EXTI9_5, local = [last: Option<Instant<InfraMono>> = None, receiver])]
    fn ir_rx(cx: ir_rx::Context) {
        let now = monotonics::InfraMono::now();
        let last = cx.local.last;
        let receiver = cx.local.receiver;

        receiver.pin().clear_interrupt_pending_bit();

        if let Some(last) = last {
            let dt = now.checked_duration_since(&last).unwrap().integer();

            if let Ok(Some(button)) = receiver.event(dt) {
                if let Some(action) = button.action() {
                    rprintln!("{:?}", button);
                    let key = super::mediakey_from_action(action);
                    rprintln!("{:?}", key);
                    keydown::spawn(key).unwrap();
                }
            }
        }

        last.replace(now);
    }

    #[task(shared = [usb_kbd])]
    fn keydown(mut cx: keydown::Context, key: MediaKey) {
        cx.shared.usb_kbd.lock(|kbd| super::send_keycode(kbd, key));

        keyup::spawn_after(Milliseconds(20_u32)).unwrap();
    }

    #[task(shared = [usb_kbd])]
    fn keyup(mut cx: keyup::Context) {
        cx.shared
            .usb_kbd
            .lock(|kbd| super::send_keycode(kbd, MediaKey::Zero));
    }
}

fn usb_poll<B: bus::UsbBus>(
    usb_dev: &mut UsbDevice<'static, B>,
    usb_kbd: &mut HIDClass<'static, B>,
) {
    usb_dev.poll(&mut [usb_kbd]);
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

fn mediakey_from_action(action: Action) -> MediaKey {
    match action {
        Action::Play_Pause => MediaKey::PlayPause,
        Action::Up => MediaKey::VolumeIncrement,
        Action::Down => MediaKey::VolumeDecrement,
        Action::Right => MediaKey::NextTrack,
        Action::Left => MediaKey::PrevTrack,
        Action::Stop => MediaKey::Stop,
        _ => MediaKey::Zero,
    }
}
