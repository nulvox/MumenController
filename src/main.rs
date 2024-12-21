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
#![feature(generic_arg_infer)]

use teensy4_panic as _;
mod usb;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use bsp::{
        hal::{adc, gpio, iomuxc},
        pins,
    };

    // use teensy4_bsp::ral::iomuxc;
    use teensy4_bsp::{self as bsp, hal::iomuxc::Pad};

    use imxrt_log as logging;

    // If you're using a Teensy 4.1 or MicroMod, you should eventually
    // change 't40' to 't41' or micromod, respectively.
    use board::t40 as my_board;

    use rtic_monotonics::systick::{Systick, *};

    use crate::usb::*;
    use embedded_hal::digital::InputPin;
    const PIN_CONFIG: iomuxc::Config =
        iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));

    #[shared]
    struct Shared {
        keys: PadReport,
    }

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        /// The LED on pin 13.
        // led: board::Led,
        /// A poller to control USB logging.
        poller: logging::Poller,
        keydata: KeyData,
        pin_a: gpio::Input<pins::t40::P14>,
        pin_b: gpio::Input<pins::t40::P11>,
        pin_x: gpio::Input<pins::t40::P9>,
        pin_y: gpio::Input<pins::t40::P16>,
        pin_l1: gpio::Input<pins::t40::P15>,
        pin_r1: gpio::Input<pins::t40::P10>,
        pin_l2: gpio::Input<pins::t40::P12>,
        pin_r2: gpio::Input<pins::t40::P13>,
        pin_l3: gpio::Input<pins::t40::P3>,
        pin_r3: gpio::Input<pins::t40::P2>,
        pin_select: gpio::Input<pins::t40::P18>,
        pin_start: gpio::Input<pins::t40::P17>,
        pin_home: gpio::Input<pins::t40::P8>,
        pin_up: gpio::Input<pins::t40::P1>,
        pin_down: gpio::Input<pins::t40::P6>,
        pin_left: gpio::Input<pins::t40::P7>,
        pin_right: gpio::Input<pins::t40::P19>,
        pin_t_analog_left: gpio::Input<pins::t40::P4>,
        pin_t_analog_right: gpio::Input<pins::t40::P5>,
        pin_lock: gpio::Input<pins::t40::P0>,
        pin_rx: gpio::Input<pins::t40::P22>,
        pin_ry: gpio::Input<pins::t40::P23>,
        pin_lx: gpio::Input<pins::t40::P20>,
        pin_ly: gpio::Input<pins::t40::P21>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio1,
            mut gpio2,
            // mut gpio3,
            mut gpio4,
            mut pins,
            usb,
            // adc1,
            // adc2,
            ..
        } = my_board(cx.device);
        iomuxc::configure(&mut pins.p0, PIN_CONFIG);
        iomuxc::configure(&mut pins.p1, PIN_CONFIG);
        iomuxc::configure(&mut pins.p2, PIN_CONFIG);
        iomuxc::configure(&mut pins.p3, PIN_CONFIG);
        iomuxc::configure(&mut pins.p4, PIN_CONFIG);
        iomuxc::configure(&mut pins.p5, PIN_CONFIG);
        iomuxc::configure(&mut pins.p6, PIN_CONFIG);
        iomuxc::configure(&mut pins.p7, PIN_CONFIG);
        iomuxc::configure(&mut pins.p8, PIN_CONFIG);
        iomuxc::configure(&mut pins.p9, PIN_CONFIG);
        iomuxc::configure(&mut pins.p10, PIN_CONFIG);
        iomuxc::configure(&mut pins.p11, PIN_CONFIG);
        iomuxc::configure(&mut pins.p12, PIN_CONFIG);
        iomuxc::configure(&mut pins.p13, PIN_CONFIG);
        iomuxc::configure(&mut pins.p14, PIN_CONFIG);
        iomuxc::configure(&mut pins.p15, PIN_CONFIG);
        iomuxc::configure(&mut pins.p16, PIN_CONFIG);
        iomuxc::configure(&mut pins.p17, PIN_CONFIG);
        iomuxc::configure(&mut pins.p18, PIN_CONFIG);
        iomuxc::configure(&mut pins.p19, PIN_CONFIG);
        iomuxc::configure(&mut pins.p20, PIN_CONFIG);
        iomuxc::configure(&mut pins.p21, PIN_CONFIG);
        iomuxc::configure(&mut pins.p22, PIN_CONFIG);
        iomuxc::configure(&mut pins.p23, PIN_CONFIG);

        // let led = board::led(&mut gpio2, pins.p13);
        let pin_a = gpio1.input(pins.p14);
        let pin_b = gpio2.input(pins.p11);
        let pin_x = gpio2.input(pins.p9);
        let pin_y = gpio1.input(pins.p16);
        let pin_l1 = gpio1.input(pins.p15);
        let pin_r1 = gpio2.input(pins.p10);
        let pin_l2 = gpio2.input(pins.p12);
        let pin_r2 = gpio2.input(pins.p13);
        let pin_l3 = gpio4.input(pins.p3);
        let pin_r3 = gpio4.input(pins.p2);
        let pin_select = gpio1.input(pins.p18);
        let pin_start = gpio1.input(pins.p17);
        let pin_home = gpio2.input(pins.p8);
        let pin_up = gpio1.input(pins.p1);
        let pin_down = gpio2.input(pins.p6);
        let pin_left = gpio2.input(pins.p7);
        let pin_right = gpio1.input(pins.p19);
        let pin_t_analog_left = gpio4.input(pins.p4);
        let pin_t_analog_right = gpio4.input(pins.p5);
        let pin_lock = gpio1.input(pins.p0);
        let pin_rx = gpio1.input(pins.p22);
        let pin_ry = gpio1.input(pins.p23);
        let pin_lx = gpio1.input(pins.p20);
        let pin_ly = gpio1.input(pins.p21);

        // The poller is an artifact from the boilerplate code.
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        let keydata: KeyData = KeyData {
            buttons: 0,
            hat: 0,
            padding: 0,
            lx: 0,
            ly: 0,
            rx: 0,
            ry: 0,
        };
        let keys: PadReport = PadReport::new(&keydata);
        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        check_input::spawn().unwrap();
        (
            Shared { keys },
            Local {
                poller,
                keydata,
                pin_a,
                pin_b,
                pin_x,
                pin_y,
                pin_l1,
                pin_r1,
                pin_l2,
                pin_r2,
                pin_l3,
                pin_r3,
                pin_select,
                pin_start,
                pin_home,
                pin_up,
                pin_down,
                pin_left,
                pin_right,
                pin_t_analog_left,
                pin_t_analog_right,
                pin_lock,
                pin_rx,
                pin_ry,
                pin_lx,
                pin_ly,
            },
        )
    }

    #[task(shared = [ keys ], local = [ keydata, pin_a, pin_b, pin_x, pin_y, pin_l1, pin_r1, pin_l2, pin_r2, pin_l3, pin_r3, pin_select, pin_start, pin_home, pin_up, pin_down, pin_left, pin_right, pin_t_analog_left, pin_t_analog_right, pin_lock, pin_rx, pin_ry, pin_lx, pin_ly ])]
    async fn check_input(mut cx: check_input::Context) {
        loop {
            //     if cx.local.pin_t_analog_left.is_low().unwrap() {
            //         // Do some stuff with the analog stick
            //         pass
            //     } else if cx.local.pin_t_analog_right.is_high().unwrap() {
            //         // cx.shared.keydata.lx = 255;
            //         // Do some stuff with the analog stick
            //         pass
            //     }
            // cx.shared.keys.clear_keys(&self);
            // Access keys through cx.shared
            cx.shared.keys.lock(|keys| {
                keys.clear_keys();
                // lets keep the closure open until we set all the keys.
                // this prevents the system from generating a race condition with `keys`
                // });
                if cx.local.pin_a.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_A;
                }
                if cx.local.pin_b.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_B;
                }
                if cx.local.pin_x.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_X;
                }
                if cx.local.pin_y.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_Y;
                }
                if cx.local.pin_l1.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_L1;
                }
                if cx.local.pin_r1.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_R1;
                }
                if cx.local.pin_l2.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_L2;
                }
                if cx.local.pin_r2.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_R2;
                }
                if cx.local.pin_l3.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_L3;
                }
                if cx.local.pin_r3.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_R3;
                }
                if cx.local.pin_select.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_SELECT;
                }
                if cx.local.pin_start.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_START;
                }
                if cx.local.pin_home.is_low().unwrap() {
                    cx.local.keydata.buttons |= KEY_MASK_HOME;
                }

                // rewrite the logic for the hat switch
                if cx.local.pin_up.is_low().unwrap() {
                    // masks are your friend here
                    cx.local.keydata.hat |= HAT_MASK_UP;
                }
                if cx.local.pin_down.is_low().unwrap() {
                    cx.local.keydata.hat |= HAT_MASK_DOWN;
                }
                if cx.local.pin_left.is_low().unwrap() {
                    cx.local.keydata.hat |= HAT_MASK_LEFT;
                }
                if cx.local.pin_right.is_low().unwrap() {
                    cx.local.keydata.hat |= HAT_MASK_RIGHT;
                }

                if cx.local.pin_t_analog_left.is_low().unwrap() {
                    // We should measure the analog stick and set the value
                    cx.local.keydata.lx = 255;
                    cx.local.keydata.ly = 255;
                }
                if cx.local.pin_t_analog_right.is_low().unwrap() {
                    // We should measure the analog stick and set the value
                    cx.local.keydata.rx = 255;
                    cx.local.keydata.ry = 255;
                }
            });
        }
    }

    #[task(binds = USB_OTG1, local = [ poller ])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
