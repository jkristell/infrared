#![no_std]
#![no_main]

// How to run this example (in the infrared/examples/rp2040 directory):
//  cargo run --release --bin receiver-event-driven-rtic

use panic_halt as _;
use rp2040_hal as hal;

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[rtic::app(device = crate::hal::pac, peripherals = true, dispatchers = [PIO0_IRQ_0])]
mod app {
    use core::fmt::Write;
    use hal::clocks::init_clocks_and_plls;
    use hal::gpio::bank0::Gpio12;
    use hal::gpio::{Floating, Input, Interrupt, Pin};
    use hal::sio::Sio;
    use hal::watchdog::Watchdog;
    use rp2040_hal as hal;
    use rtt_target::rtt_init;
    use rtt_target::UpChannel;

    // IR receiver stuff
    use infrared::protocol::*;
    use infrared::receiver::Receiver;

    // Useful aliases
    pub type IRPin = Pin<Gpio12, Input<Floating>>;
    pub type IrReceiver = Receiver<Nec, IRPin>;

    #[shared]
    struct Shared {
        timer: hal::timer::Timer,
        #[lock_free]
        debug_ch0: UpChannel,
        ir_recv: IrReceiver,
        ir_edge: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut watchdog = Watchdog::new(c.device.WATCHDOG);
        let mut resets = c.device.RESETS;
        let _clocks = init_clocks_and_plls(
            12_000_000u32,
            c.device.XOSC,
            c.device.CLOCKS,
            c.device.PLL_SYS,
            c.device.PLL_USB,
            // &mut pac.RESETS,
            &mut resets,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let sio = Sio::new(c.device.SIO);
        // Set the pins to their default state
        let pins = hal::gpio::Pins::new(
            c.device.IO_BANK0,
            c.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut resets,
        );

        // Setup the IR stuff
        let ir_sensor_pin = pins.gpio12.into_floating_input();
        ir_sensor_pin.set_interrupt_enabled(Interrupt::EdgeLow, true);
        ir_sensor_pin.set_interrupt_enabled(Interrupt::EdgeHigh, true);
        let ir_recv = Receiver::with_pin(1_000_000, ir_sensor_pin);

        // Used for the IR interrupt high/low edge tracking:
        let ir_edge: u32 = 0;

        // Delay for power on
        for _ in 0..1000 {
            cortex_m::asm::nop();
        }

        let timer = hal::Timer::new(c.device.TIMER, &mut resets);

        // Setup some rtt-target debugging:
        let rtt_channels = rtt_init! { // NOTE: DO NOT MOVE THIS HIGHER
            up: {
                0: {
                    size: 512
                    name: "Main"
                }
            }
        };
        let mut debug_ch0: UpChannel = rtt_channels.up.0;
        let _ = writeln!(debug_ch0, "init()"); // So we know everything finished initializing

        (
            Shared {
                timer,
                debug_ch0,
                ir_recv,
                ir_edge,
            },
            Local {},
            init::Monotonics(),
        )
    }

    // Our event-based interrupt handler for infrared stuff:
    #[task(
        binds = IO_IRQ_BANK0, // You "just have to know" that this interrupt is correct :)
        priority = 1,
        shared = [debug_ch0, timer, ir_recv, ir_edge],
        local = [],
    )]
    fn do_ir(c: do_ir::Context) {
        let debug_ch0 = c.shared.debug_ch0;
        (c.shared.timer, c.shared.ir_recv, c.shared.ir_edge).lock(|t, ir, ire| {
            // Clear the interrupts
            let pin = ir.pin_mut();
            pin.clear_interrupt(Interrupt::EdgeLow);
            pin.clear_interrupt(Interrupt::EdgeHigh);
            let now = t.get_counter_low();
            let dt = now.wrapping_sub(*ire);
            *ire = now;
            // Write out our debug info
            if let Ok(Some(cmd)) = ir.event(dt) {
                let _ = writeln!(debug_ch0, "do_ir() {:?}", cmd);
            }
        });
    }
}
