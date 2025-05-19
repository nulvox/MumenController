//! Demonstrates a USB keypress using RTIC.
//!
//! Flash your board with this example. Your device will occasionally
//! send some kind of keypress to your host.

#![no_std]
#![no_main]

// Custom panic handler
mod panic_handler;
mod spc;
mod pinouts;
// Cannot use std::boxed::Box in a no_std environment
// We'll use a different approach

// Remove the imports and use direct paths to the panic_handler functions

#[rtic::app(device = teensy4_bsp, peripherals = true)]
mod app {
    use hal::usbd::{BusAdapter, EndpointMemory, EndpointState, Speed};
    use imxrt_hal as hal;
    use usbd_hid_device::HidReport;  // Import HidReport trait
    use crate::pinouts;

    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
    };
    use usbd_hid::hid_class::HIDClass;
    use teensy4_bsp as bsp;
    use bsp::{
        hal::{gpio, iomuxc},
    };



    /// USB Speed configuration - High speed for better performance
    const SPEED: Speed = Speed::High;
    /// Use a generic VID/PID that won't conflict with existing devices
    /// Use a generic VID/PID that won't conflict with existing devices
    /// This is a test VID/PID - you should replace with your own for production
    const VID_PID: UsbVidPid = UsbVidPid(0x1209, 0x0001);
    const PRODUCT: &str = "mumen";
    /// How frequently should we poll the logger?
    /// // @TODO, what is the resolution here? This is the default value given to us by the example.
    const LPUART_POLL_INTERVAL_MS: u32 = 200; // Increased from 100ms to 200ms to reduce power consumption
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


    use crate::spc::{self, PadReport, KeyData};
    use crate::pinouts::{PinoutConfig, PinType, PinConfig, is_pin_low, is_pin_high};
    // use teensy4_bsp::hal::iomuxc::adc::Pin as AdcPin;
    // Use 22k pull-up resistors for better power efficiency
    const PIN_CONFIG: iomuxc::Config =
    iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup100k));
    #[local]
    struct Local {
        hid: HIDClass<'static, Bus>,
        device: UsbDevice<'static, Bus>,
        // poller: board::logging::Poller,
        timer: hal::pit::Pit<0>,
        // message: MessageIter,
        keydata: KeyData,
        pins: PinConfig,
        pinout: &'static dyn PinoutConfig,
    }

    #[shared]
    struct Shared {
        keys: PadReport,
    }

    // Minimal initialization focused ONLY on LED testing
    #[init(local = [bus: Option<UsbBusAllocator<Bus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Set section to initialization
        crate::panic_handler::set_code_section(crate::panic_handler::SECTION_INIT);
        crate::panic_handler::clear_error_flag();
        
        // PURE LED TEST FIRMWARE
        // Initialize only what's absolutely required
        let device = ctx.device;
        let board = bsp::board::t40(device);
        let timer = board.pit.0;
        
        // Set up the minimal USB requirements to avoid hardware issues
        let bus = ctx.local.bus.insert(UsbBusAllocator::new(
            BusAdapter::new(board.usb, &EP_MEMORY, &EP_STATE)
        ));
        
        // Create minimal HID class with basic settings
        let hid = HIDClass::new(
            bus,
            spc::KeyData::DESCRIPTOR,
            50, // Long polling interval to save power
        );
        
        // Create a minimal device that just needs to exist
        let device = UsbDeviceBuilder::new(bus, VID_PID)
            .manufacturer("Mumen")
            .product("Controller")
            .serial_number("1")
            .device_class(0)
            .build();
        
        // Create the pinout configuration
        let pinout: &'static dyn PinoutConfig = unsafe {
            let trait_obj: &dyn PinoutConfig = &pinouts::create_pinout();
            core::mem::transmute::<&dyn PinoutConfig, &'static dyn PinoutConfig>(trait_obj)
        };
        
        // Create empty keydata
        let keydata = spc::KeyData::default();
        
        // Create empty pins config
        let pins_config = pinouts::PinConfig {
            active_pins: 0,
        };
        
        // Create dummy keys
        let keys = spc::PadReport::new(&keydata);
        
        // DIRECT LED BLINKING TEST - NO ASYNC TASKS
        // Run LED test directly in init to ensure it works
        crate::panic_handler::set_code_section(crate::panic_handler::SECTION_LED_TEST);
        
        // Define all the GPIO bases we want to try
        let gpio_bases = [
            0x401B8000u32, // GPIO1
            0x401BC000,    // GPIO2
            0x401C0000,    // GPIO3
            0x401C4000,    // GPIO4
            0x400C0000,    // GPIO5
            0x42000000,    // GPIO6 (most common for the LED)
            0x42004000,    // GPIO7
            0x4200C000,    // GPIO9
        ];
        
        // Loop forever, trying different GPIO pins
        // The goal is to find ANY working LED configuration
        loop {
            for &gpio_base in gpio_bases.iter() {
                // Try pins 3 and 13 - the most common for LED
                for &pin in [3u8, 13].iter() {
                    let led_mask = 1u32 << pin;
                    
                    unsafe {
                        let gpio_base_ptr = gpio_base as *mut u32;
                        let gdir = gpio_base_ptr.offset(1); // Direction register
                        let dr = gpio_base_ptr.offset(0);   // Data register
                        
                        // Configure pin as output
                        let current_gdir = core::ptr::read_volatile(gdir);
                        core::ptr::write_volatile(gdir, current_gdir | led_mask);
                        
                        // Blink pattern: ON-OFF-ON-OFF-ON (SOS start)
                        for _ in 0..5 {
                            // LED ON
                            let current_dr = core::ptr::read_volatile(dr);
                            core::ptr::write_volatile(dr, current_dr | led_mask);
                            
                            // Delay 500ms
                            for _ in 0..2000000 { core::hint::spin_loop(); }
                            
                            // LED OFF
                            let current_dr = core::ptr::read_volatile(dr);
                            core::ptr::write_volatile(dr, current_dr & !led_mask);
                            
                            // Delay 500ms
                            for _ in 0..2000000 { core::hint::spin_loop(); }
                        }
                        
                        // After blinking, if this is GPIO6 pin 13 (most likely to work),
                        // test the panic handler by deliberately causing a panic
                        if gpio_base == 0x42000000 && pin == 13 {
                            // Set different code sections and error flags to test panic display
                            crate::panic_handler::set_code_section(crate::panic_handler::SECTION_HID_REPORT);
                            crate::panic_handler::set_error_flag(crate::panic_handler::ERR_HID_REPORT);
                            
                            // Trigger panic to test panic handler with reporting
                            panic!("Testing panic handler with section={} error={}",
                                crate::panic_handler::SECTION_HID_REPORT,
                                crate::panic_handler::ERR_HID_REPORT);
                        }
                    }
                }
            }
        }
        
        // Return minimal structs
        (
            Shared { keys },
            Local {
                hid,
                device,
                timer,
                keydata,
                pins: pins_config,
                pinout,
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

    // LED test is now run directly inside init instead of as a task

    #[task(binds = USB_OTG1, shared = [keys], local = [device, hid, configured: bool = false, last_report: [u8; 8] = [0; 8], poll_counter: u32 = 0], priority = 2)]
    fn usb1(mut ctx: usb1::Context) {
        // Ultra-minimal USB task - just basic polling for the LED test firmware
        
        // Set section to USB polling in panic handler
        crate::panic_handler::set_code_section(crate::panic_handler::SECTION_USB_POLL);
        crate::panic_handler::clear_error_flag();
        
        // Very minimal USB polling - just to avoid hanging
        let usb1::LocalResources {
            hid,
            device,
            configured,
            last_report,
            ..
        } = ctx.local;

        // Just poll USB with minimal operations
        let _ = device.poll(&mut [hid]);
        
        // Only do basic state tracking
        if device.state() == usb_device::device::UsbDeviceState::Configured {
            if !*configured {
                *configured = true;
            }
            
            // Only do minimal report sending to avoid issues
            ctx.shared.keys.lock(|keys| {
                let curr_report = keys.as_ref();
                if curr_report != *last_report {
                    last_report.copy_from_slice(curr_report);
                    let _ = hid.push_input(keys);
                }
            });
        } else if *configured {
            *configured = false;
        }
    }

    // Removed check_input task - we're focusing only on the LED test
}
