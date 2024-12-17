//! The starter code slowly blinks the LED and sets up
//! USB logging. It periodically logs messages over USB.
//!
//! Despite targeting the Teensy 4.0, this starter code
//! should also work on the Teensy 4.1 and Teensy MicroMod.
//! You should eventually target your board! See inline notes.
//!
//! This template uses [RTIC v2](https://rtic.rs/2/book/en/)
//! for structuring the application.

#![no_std]
#![no_main]

use teensy4_panic as _;
mod input;
mod usb;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use teensy4_bsp as bsp;

    use imxrt_log as logging;

    // If you're using a Teensy 4.1 or MicroMod, you should eventually
    // change 't40' to 't41' or micromod, respectively.
    use board::t40 as my_board;

    use rtic_monotonics::systick::{Systick, *};

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        /// The LED on pin 13.
        led: board::Led,
        /// A poller to control USB logging.
        poller: logging::Poller,
        /// The USB peripheral.
        usb: bsp::usb::UsbBus,
        /// Digital Input Pins
        pins: bsp::t40::Pins,
        /// Analog Input Pins
        analog_pins: bsp::t40::AnalogPins,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            pins,
            usb,
            ..
        } = my_board(cx.device);

        // let led = board::led(&mut gpio2, pins.p13);
        let pin_a = SWITCH_A;
        let pin_b = SWITCH_B;
        let pin_x = SWITCH_X;
        let pin_y = SWITCH_Y;
        let pin_l1 = SWITCH_L1;
        let pin_r1 = SWITCH_R1;
        let pin_l2 = SWITCH_L2;
        let pin_r2 = SWITCH_R2;
        let pin_select = SWITCH_SELECT;
        let pin_start = SWITCH_START;
        let pin_home = SWITCH_HOME;
        let pin_shift = SWITCH_SHIFT;
        let pin_up = SWITCH_UP;
        let pin_down = SWITCH_DOWN;
        let pin_left = SWITCH_LEFT;
        let pin_right = SWITCH_RIGHT;
        let pin_t_analog_left = SWITCH_T_ANALOG_LEFT;
        let pin_t_analog_right = SWITCH_T_ANALOG_RIGHT;
        let pin_lockout = SWITCH_LOCKOUT;

        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        (Shared {}, Local { led, poller })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let mut count = 0u32;
        loop {
            cx.local.led.toggle();
            Systick::delay(500.millis()).await;

            log::info!("Hello from your Teensy 4! The count is {count}");
            if count % 7 == 0 {
                log::warn!("Here's a warning at count {count}");
            }
            if count % 23 == 0 {
                log::error!("Here's an error at count {count}");
            }

            count = count.wrapping_add(1);
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
