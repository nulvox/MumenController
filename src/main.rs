//! Demonstrates a USB keypress using RTIC.
//!
//! Flash your board with this example. Your device will occasionally
//! send some kind of keypress to your host.

#![no_std]
#![no_main]
#![feature(generic_arg_infer)]

use teensy4_panic as _;
mod spc;

#[rtic::app(device = board, peripherals = false)]
mod app {
    use hal::usbd::{BusAdapter, EndpointMemory, EndpointState, Speed};
    use imxrt_hal as hal;

    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
    };
    use usbd_hid::{
        descriptor::{KeyboardReport, SerializedDescriptor as _},
        hid_class::HIDClass,
    };
    use teensy4_bsp::{self as bsp};
    use board::t40 as my_board;
    use bsp::board;
    use bsp::{
        hal::{gpio, iomuxc},
        hal::{gpio, iomuxc, usbd},
        // hal::{adc, gpio, iomuxc, usbd},
        pins,
    };



    /// Change me if you want to play with a full-speed USB device.
    const SPEED: Speed = Speed::full;
    /// Taken from Switch Pro Controller lsusb
    const VID_PID: UsbVidPid = UsbVidPid(0x057E, 0x2009);
    const PRODUCT: &str = "mumen";
    /// How frequently should we poll the logger?
    /// // @TODO, what is the resolution here? This is the default value given to us by the example.
    const LPUART_POLL_INTERVAL_MS: u32 = board::PIT_FREQUENCY / 1_000 * 100;
    /// Change me to change how log messages are serialized.
    ///
    /// If changing to `Defmt`, you'll need to update the logging macros in
    /// this example. You'll also need to make sure the USB device you're debugging
    /// uses `defmt`.
    // const FRONTEND: board::logging::Frontend = board::logging::Frontend::Log;
    /// The USB GPT timer we use to (infrequently) send mouse updates.
    // const GPT_INSTANCE: imxrt_usbd::gpt::Instance = imxrt_usbd::gpt::Instance::Gpt0;
    /// How frequently should we push mouse updates to the host?
    // const MOUSE_UPDATE_INTERVAL_MS: u32 = 200;

    /// This allocation is shared across all USB endpoints. It needs to be large
    /// enough to hold the maximum packet size for *all* endpoints. If you start
    /// noticing panics, check to make sure that this is large enough for all endpoints.
    static EP_MEMORY: EndpointMemory<1024> = EndpointMemory::new();
    /// This manages the endpoints. It's large enough to hold the maximum number
    /// of endpoints; we're not using all the endpoints in this example.
    static EP_STATE: EndpointState = EndpointState::max_endpoints();

    type Bus = BusAdapter;

    // use adc::AnalogInput;
    use embedded_hal::digital::InputPin;

    use crate::spc;
    // use teensy4_bsp::hal::iomuxc::adc::Pin as AdcPin;
    const PIN_CONFIG: iomuxc::Config =
    iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pulldown100k));
    #[local]
    struct Local {
        hid: HIDClass<'static, Bus>,
        device: UsbDevice<'static, Bus>,
        led: board::Led,
        poller: board::logging::Poller,
        timer: hal::pit::Pit<0>,
        message: MessageIter,
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
        // pin_rx: adc::AnalogInput<pins::t40::P22, 9>,
        // pin_ry: adc::AnalogInput<pins::t40::P23, 10>,
        // pin_lx: adc::AnalogInput<pins::t40::P20, 7>,
        // pin_ly: adc::AnalogInput<pins::t40::P21, 8>,
    }

    #[shared]
    struct Shared {
        keys: PadReport,
    }

    #[init(local = [bus: Option<UsbBusAllocator<Bus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let (
            board::Common {
                pit: (mut timer, _, _, _),
                usb1,
                usbnc1,
                usbphy1,
                mut dma,
                mut gpio1,
                mut gpio2,
                mut gpio4,
                mut pins,
                ..
            },
            board::Specifics { led, console, .. },
        ) = board::new();
        // configure GPIO for the buttons
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
        // iomuxc::configure(&mut pins.p20, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p21, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p22, ANALOG_PIN_CONFIG);
        // iomuxc::configure(&mut pins.p23, ANALOG_PIN_CONFIG);

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
        // let pin_rx: adc::AnalogInput<pins::t40::P22, 9> = adc::AnalogInput::new(pins.p22);
        // let pin_ry: adc::AnalogInput<pins::t40::P23, 10> = adc::AnalogInput::new(pins.p23);
        // let pin_lx: adc::AnalogInput<pins::t40::P20, 7> = adc::AnalogInput::new(pins.p20);
        // let pin_ly: adc::AnalogInput<pins::t40::P21, 8> = adc::AnalogInput::new(pins.p21);

        timer.set_load_timer_value(LPUART_POLL_INTERVAL_MS);
        timer.set_interrupt_enable(true);
        timer.enable();

        // let dma_a = dma[board::BOARD_DMA_A_INDEX].take().unwrap();
        // let poller = board::logging::lpuart(FRONTEND, console, dma_a);

        let usbd = hal::usbd::Instances {
            usb: usb1,
            usbnc: usbnc1,
            usbphy: usbphy1,
        };

        let bus = BusAdapter::with_speed(usbd, &EP_MEMORY, &EP_STATE, SPEED);
        bus.set_interrupts(true);
        // bus.gpt_mut(GPT_INSTANCE, |gpt| {
        //     gpt.stop();
        //     gpt.clear_elapsed();
        //     gpt.set_interrupt_enabled(true);
        //     gpt.set_mode(imxrt_usbd::gpt::Mode::Repeat);
        //     gpt.set_load(MOUSE_UPDATE_INTERVAL_MS * 1000);
        //     gpt.reset();
        //     gpt.run();
        // });

        let bus = ctx.local.bus.insert(UsbBusAllocator::new(bus));
        // Note that "4" correlates to a 1ms polling interval. Since this is a high speed
        // device, bInterval is computed differently.
        let class = HIDClass::new(bus, KeyboardReport::desc(), 4);
        // @here, configure the usb-hid to use the one in usb.rs
        //  the line above these comments likely neeeds to point at the types
        //    which we define in usb.rs...
        let device = UsbDeviceBuilder::new(bus, VID_PID)
            .strings(&[usb_device::device::StringDescriptors::default().product(PRODUCT)])
            .unwrap()
            .device_class(usbd_hid::hid_class)
            .max_packet_size_0(64)
            .unwrap()
            .build();

        (
            Shared { keys },
            Local {
                hid,
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
                class,
                device,
                // led,
                // poller,
                // timer,
                // message: MESSAGE.iter().cycle(),
            },
        );
        check_input::spawn().unwrap();
        (
            Shared { keys },
            Local {
                hid,
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
                // pin_rx,
                // pin_ry,
                // pin_lx,
                // pin_ly,
            },
        )
    }

    // #[task(binds = BOARD_PIT, local = [poller, timer], priority = 1)]
    // fn pit_interrupt(ctx: pit_interrupt::Context) {
    //     while ctx.local.timer.is_elapsed() {
    //         ctx.local.timer.clear_elapsed();
    //     }

    //     ctx.local.poller.poll();
    // }

    #[task(binds = BOARD_USB1, local = [device, hid, configured: bool = false], priority = 2)]
    fn usb1(ctx: usb1::Context) {
        let usb1::LocalResources {
            hid,
            device,
            // led,
            configured,
            // message,
            ..
        } = ctx.local;

        device.poll(&mut [class]);

        if device.state() == UsbDeviceState::Configured {
            if !*configured {
                device.bus().configure();
            }
            *configured = true;
        } else {
            *configured = false;
        }

        if *configured {
            // let elapsed = device.bus().gpt_mut(GPT_INSTANCE, |gpt| {
            //     let elapsed = gpt.is_elapsed();
            //     while gpt.is_elapsed() {
            //         gpt.clear_elapsed();
            //     }
            //     elapsed
            // });

            // if elapsed {
                // led.toggle();
                // let code = *message.next().unwrap();
                // if let Some(report) = translate_char(code) {
                    class.push_input(&report).ok();
                // }
                // @TODO this is where we pushed a char after 
                //  the wait was done in the weird eldritch keybaord example...
            // }
        }
    }

    #[task(shared = [ keys ], local = [ 
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
        // pin_rx, 
        // pin_ry, 
        // pin_lx, 
        // pin_ly 
        ])] 
    async fn check_input(mut cx: check_input::Context) {
        let mut dpad: u8 = 0;
        loop {
            dpad = 0;
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
                if cx.local.pin_a.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_A;
                }
                if cx.local.pin_b.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_B;
                }
                if cx.local.pin_x.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_X;
                }
                if cx.local.pin_y.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_Y;
                }
                if cx.local.pin_l1.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L1;
                }
                if cx.local.pin_r1.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R1;
                }
                if cx.local.pin_l2.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L2;
                }
                if cx.local.pin_r2.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R2;
                }
                if cx.local.pin_l3.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L3;
                }
                if cx.local.pin_r3.is_low().unwrap() {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R3;
                }
                if cx.local.pin_lock.is_high().unwrap() {
                    if cx.local.pin_select.is_low().unwrap() {
                        cx.local.keydata.buttons |= spc::KEY_MASK_SELECT;
                    }
                    if cx.local.pin_start.is_low().unwrap() {
                        cx.local.keydata.buttons |= spc::KEY_MASK_START;
                    }
                    if cx.local.pin_home.is_low().unwrap() {
                        cx.local.keydata.buttons |= spc::KEY_MASK_HOME;
                    }
                    // If we add a capture pin, it would go here... 
                }
                // Digital processing of analog sticks
                // AnalogStick toggle set to left stick
                if cx.local.pin_t_analog_left.is_low().unwrap() {
                    if cx.local.pin_down.is_low().unwrap() {
                        if cx.local.pin_up.is_low().unwrap() {
                            cx.local.keydata.ly = 255;
                        }
                        else {
                            cx.local.keydata.ly = 64;
                        }
                    }
                    else if cx.local.pin_down.is_low().unwrap() {
                        cx.local.keydata.ly = 0;
                            
                    }
                    if cx.local.pin_left.is_low().unwrap() {
                        if cx.local.pin_right.is_low().unwrap() {
                            cx.local.keydata.lx = 64;
                        }
                        else{
                            cx.local.keydata.lx = 0;
                        }
                    }
                    else if cx.local.pin_right.is_low().unwrap() {
                        cx.local.keydata.lx = 255;
                    }
                }
                // AnalogStick toggle set to right stick
                else if cx.local.pin_t_analog_right.is_low().unwrap() {
                    if cx.local.pin_down.is_low().unwrap() {
                        if cx.local.pin_up.is_low().unwrap() {
                            cx.local.keydata.ry = 255;
                        }
                        else {
                            cx.local.keydata.ry = 64;
                        }
                    }
                    else if cx.local.pin_down.is_low().unwrap() {
                        cx.local.keydata.ry = 0;
                            
                    }
                    if cx.local.pin_left.is_low().unwrap() {
                        if cx.local.pin_right.is_low().unwrap() {
                            cx.local.keydata.rx = 64;
                        }
                        else{
                            cx.local.keydata.rx = 0;
                        }
                    }
                    else if cx.local.pin_right.is_low().unwrap() {
                        cx.local.keydata.rx = 255;
                    }
                // AS toggle not set, process D-Pad
                } else {
                    // Check up and down, clean SOCD
                    if cx.local.pin_down.is_low().unwrap() {
                        if cx.local.pin_up.is_low().unwrap() {
                            dpad |= spc::HAT_MASK_UP;
                        }
                        else {
                            dpad |= spc::HAT_MASK_DOWN;
                        }
                    }
                    else if cx.local.pin_down.is_low().unwrap() {
                        dpad |= spc::HAT_MASK_DOWN;
                            
                    }
                    // NOW LEFT AND RIGHT, still cleaning
                    if cx.local.pin_left.is_low().unwrap() {
                        dpad |= spc::HAT_MASK_LEFT;
                    }
                    else if cx.local.pin_right.is_low().unwrap() {
                        dpad |= spc::HAT_MASK_RIGHT;
                    }
                    keys.set_hat(dpad);
                }
            });
        }
    }
}
