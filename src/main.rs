//! Demonstrates a USB keypress using RTIC.
//!
//! Flash your board with this example. Your device will occasionally
//! send some kind of keypress to your host.

#![no_std]
#![no_main]

use teensy4_panic as _;
mod spc;
mod pinouts;

#[rtic::app(device = teensy4_bsp, peripherals = true)]
mod app {
    use hal::usbd::{BusAdapter, EndpointMemory, EndpointState, Speed};
    use imxrt_hal as hal;
    use usbd_hid_device::HidReport;  // Import HidReport trait

    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
    };
    use usbd_hid::hid_class::HIDClass;
    use teensy4_bsp as bsp;
    use bsp::{
        // hal::{gpio, iomuxc},
        hal::{gpio, iomuxc},
        // hal::{adc, gpio, iomuxc, usbd},
        pins,
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

    // use adc::AnalogInput;
    use embedded_hal::digital::InputPin;

    use crate::spc::{self, PadReport, KeyData};
    use crate::pinouts::{PinoutConfig, PinType, PinConfig, is_pin_low, is_pin_high};
    // use teensy4_bsp::hal::iomuxc::adc::Pin as AdcPin;
    // Use 22k pull-up resistors for better power efficiency
    const PIN_CONFIG: iomuxc::Config =
    iomuxc::Config::zero().set_pull_keeper(Some(iomuxc::PullKeeper::Pullup22k));
    #[local]
    struct Local {
        hid: HIDClass<'static, Bus>,
        device: UsbDevice<'static, Bus>,
        // poller: board::logging::Poller,
        timer: hal::pit::Pit<0>,
        // message: MessageIter,
        keydata: KeyData,
        pins: PinConfig,
        pinout: Box<dyn PinoutConfig>,
    }

    #[shared]
    struct Shared {
        keys: PadReport,
    }

    #[init(local = [bus: Option<UsbBusAllocator<Bus>> = None])]
    fn init(ctx: init::Context) -> (Shared, Local) {
        // Initialize the board
        // Get the hardware peripherals
        let peripherals = ctx.device;
        
        // Initialize the board
        let board = bsp::board::t40(peripherals);
        
        // Get pins and components
        // Extract components from the Resources struct
        let mut pins = board.pins;
        let mut timer = board.pit.0;
        let mut gpio1 = board.gpio1;
        let mut gpio2 = board.gpio2;
        let mut gpio4 = board.gpio4;
        
        // GPIOs are initialized by the BSP, no need for manual clock enabling
        
        // Configure timer
        timer.set_load_timer_value(LPUART_POLL_INTERVAL_MS);
        timer.set_interrupt_enable(true);
        timer.enable();
        
        // Create pinout configuration based on selected feature
        let pinout: Box<dyn PinoutConfig> = Box::new(pinouts::create_pinout());
        
        // Use the USB directly
        let usbd = board.usb;
        
        // Configure pins using the pinout configuration
        let pins_config = pinout.configure_pins(&mut pins, &mut gpio1, &mut gpio2, &mut gpio4);

        timer.set_load_timer_value(LPUART_POLL_INTERVAL_MS);
        timer.set_interrupt_enable(true);
        timer.enable();

        // Configure the USB bus
        let bus = BusAdapter::new(usbd, &EP_MEMORY, &EP_STATE);
        // Note: SPEED constant is defined but the BusAdapter doesn't have a set_speed method
        // The speed is configured through other means in the USB stack
        bus.set_interrupts(true);
        
        // Ensure proper bus allocation
        let bus = ctx.local.bus.insert(UsbBusAllocator::new(bus));
        // Use a more conservative polling interval to reduce power consumption
        // The dim power light suggests we might be drawing too much power
        // Increase polling interval to further reduce power consumption
        let polling_interval = 20; // Changed from 10ms to 20ms for better power efficiency
        
        // Use the KeyData descriptor from spc.rs
        let hid = HIDClass::new(bus, spc::KeyData::DESCRIPTOR, polling_interval);
        
        // Configure the USB device with power-conscious settings
        let device = UsbDeviceBuilder::new(bus, VID_PID)
            .manufacturer("Mumen Industries")
            .product("Mumen Controller")
            .serial_number("12345")
            .max_packet_size_0(64)
            .build();
// Initialize KeyData with default values and neutral positions for analog sticks
let keydata = spc::KeyData {
    buttons: 0,
    hat: 0,
    padding: 0,
    lx: pinout.get_neutral_value(PinType::Lx),
    ly: pinout.get_neutral_value(PinType::Ly),
    rx: pinout.get_neutral_value(PinType::Rx),
    ry: pinout.get_neutral_value(PinType::Ry),
};
        
        // Create a new PadReport from the KeyData
        let mut keys = spc::PadReport::new(&keydata);
        
        // Clear the keys initially
        keys.clear_keys();
        
        check_input::spawn().unwrap();
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

    #[task(binds = USB_OTG1, shared = [keys], local = [device, hid, configured: bool = false, last_report: [u8; 8] = [0; 8]], priority = 2)]
    fn usb1(mut ctx: usb1::Context) {
        let usb1::LocalResources {
            hid,
            device,
            configured,
            last_report,
            ..
        } = ctx.local;

        // Poll the USB device to process any pending events
        // This returns a boolean indicating if there was any activity
        let poll_result = device.poll(&mut [hid]);

        // Check if the device is configured
        let current_state = device.state();
        
        if current_state == UsbDeviceState::Configured {
            // Only configure once when the state changes to Configured
            if !*configured {
                device.bus().configure();
                *configured = true;
            }
            
            // Send reports regularly to ensure controller is recognized
            ctx.shared.keys.lock(|keys| {
                // Get current report to compare
                let curr_report = keys.as_ref();
                
                // Send on both conditions: if report changed OR every few iterations
                let report_changed = curr_report != *last_report;
                
                if report_changed {
                    // Store the current report
                    last_report.copy_from_slice(curr_report);
                    
                    // Simple error handling
                    if let Err(_) = hid.push_input(keys) {
                        // If push fails, try reconfiguring
                        device.bus().configure();
                    }
                }
            });
        } else if *configured {
            // Update our state tracking when device becomes unconfigured
            *configured = false;
        }
    }

    #[task(shared = [ keys ], local = [
        keydata,
        pins,
        pinout,
        ])] 
    async fn check_input(mut cx: check_input::Context) {
        let mut dpad: u8;
        
        // Debug info - initial state check
        let mut initial_state = 0u16;
        
        // Test each pin at startup, only checking configured pins
        if is_pin_low(&cx.local.pins.pin_a) && cx.local.pinout.is_configured(PinType::A) {
            initial_state |= 0x0001;
        }
        if is_pin_low(&cx.local.pins.pin_b) && cx.local.pinout.is_configured(PinType::B) {
            initial_state |= 0x0002;
        }
        if is_pin_low(&cx.local.pins.pin_x) && cx.local.pinout.is_configured(PinType::X) {
            initial_state |= 0x0004;
        }
        if is_pin_low(&cx.local.pins.pin_y) && cx.local.pinout.is_configured(PinType::Y) {
            initial_state |= 0x0008;
        }
        // Set initial buttons state to show connected
        cx.local.keydata.buttons = initial_state;
        
        loop {
            dpad = 0;
            // Access keys through cx.shared
            cx.shared.keys.lock(|keys| {
                keys.clear_keys();
                // Reset buttons for this iteration
                cx.local.keydata.buttons = 0;
                
                // Check buttons based on configuration
                
                // A and B buttons
                if cx.local.pinout.is_configured(PinType::A) && is_pin_low(&cx.local.pins.pin_a) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_A;
                }
                if cx.local.pinout.is_configured(PinType::B) && is_pin_low(&cx.local.pins.pin_b) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_B;
                }
                
                // X and Y buttons
                if cx.local.pinout.is_configured(PinType::X) && is_pin_low(&cx.local.pins.pin_x) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_X;
                }
                if cx.local.pinout.is_configured(PinType::Y) && is_pin_low(&cx.local.pins.pin_y) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_Y;
                }
                
                // Shoulder buttons
                if cx.local.pinout.is_configured(PinType::L1) && is_pin_low(&cx.local.pins.pin_l1) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L1;
                }
                if cx.local.pinout.is_configured(PinType::R1) && is_pin_low(&cx.local.pins.pin_r1) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R1;
                }
                
                // Trigger buttons (L2, R2) - only if configured
                if cx.local.pinout.is_configured(PinType::L2) && is_pin_low(&cx.local.pins.pin_l2) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L2;
                }
                if cx.local.pinout.is_configured(PinType::R2) && is_pin_low(&cx.local.pins.pin_r2) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R2;
                }
                
                // Thumbstick buttons (L3, R3) - only if configured
                if cx.local.pinout.is_configured(PinType::L3) && is_pin_low(&cx.local.pins.pin_l3) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_L3;
                }
                if cx.local.pinout.is_configured(PinType::R3) && is_pin_low(&cx.local.pins.pin_r3) {
                    cx.local.keydata.buttons |= spc::KEY_MASK_R3;
                }
                
                // Lock only if configured
                let lock_active = cx.local.pinout.is_configured(PinType::Lock) &&
                                  is_pin_high(&cx.local.pins.pin_lock);
                
                
                // Handle lock-dependent buttons (Select, Start, Home) - only if lock is active
                if lock_active {
                    if cx.local.pinout.is_configured(PinType::Select) && is_pin_low(&cx.local.pins.pin_select) {
                        cx.local.keydata.buttons |= spc::KEY_MASK_SELECT;
                    }
                    if cx.local.pinout.is_configured(PinType::Start) && is_pin_low(&cx.local.pins.pin_start) {
                        cx.local.keydata.buttons |= spc::KEY_MASK_START;
                    }
                    if cx.local.pinout.is_configured(PinType::Home) && is_pin_low(&cx.local.pins.pin_home) {
                        cx.local.keydata.buttons |= spc::KEY_MASK_HOME;
                    }
                }
                // Get directional inputs
                let t_analog_left = cx.local.pinout.is_configured(PinType::AnalogLeft) &&
                                    is_pin_low(&cx.local.pins.pin_t_analog_left);
                let t_analog_right = cx.local.pinout.is_configured(PinType::AnalogRight) &&
                                     is_pin_low(&cx.local.pins.pin_t_analog_right);
                let up_pressed = cx.local.pinout.is_configured(PinType::Up) &&
                                 is_pin_low(&cx.local.pins.pin_up);
                let down_pressed = cx.local.pinout.is_configured(PinType::Down) &&
                                   is_pin_low(&cx.local.pins.pin_down);
                let left_pressed = cx.local.pinout.is_configured(PinType::Left) &&
                                   is_pin_low(&cx.local.pins.pin_left);
                let right_pressed = cx.local.pinout.is_configured(PinType::Right) &&
                                    is_pin_low(&cx.local.pins.pin_right);
                
                // AnalogStick toggle set to left stick
                // Set analog stick values based on pinout configuration
                if cx.local.pinout.is_configured(PinType::Lx) && cx.local.pinout.is_configured(PinType::Ly) {
                    if t_analog_left {
                        // Handle Y-axis
                        if up_pressed {
                            if down_pressed {
                                cx.local.keydata.ly = 255;
                            } else {
                                cx.local.keydata.ly = 64;
                            }
                        } else if down_pressed {
                            cx.local.keydata.ly = 0;
                        } else {
                            // Ensure neutral position if no input
                            cx.local.keydata.ly = 128;
                        }
                        
                        // Handle X-axis
                        if left_pressed {
                            if right_pressed {
                                cx.local.keydata.lx = 64;
                            } else {
                                cx.local.keydata.lx = 0;
                            }
                        } else if right_pressed {
                            cx.local.keydata.lx = 255;
                        } else {
                            // Ensure neutral position if no input
                            cx.local.keydata.lx = 128;
                        }
                    }
                } else {
                    // Set neutral values for unconfigured analog sticks
                    cx.local.keydata.lx = cx.local.pinout.get_neutral_value(PinType::Lx);
                    cx.local.keydata.ly = cx.local.pinout.get_neutral_value(PinType::Ly);
                }
                // AnalogStick toggle set to right stick
                // Set right analog stick values based on pinout configuration
                if cx.local.pinout.is_configured(PinType::Rx) && cx.local.pinout.is_configured(PinType::Ry) {
                    if t_analog_right {
                        // Handle Y-axis
                        if up_pressed {
                            if down_pressed {
                                cx.local.keydata.ry = 255;
                            } else {
                                cx.local.keydata.ry = 64;
                            }
                        } else if down_pressed {
                            cx.local.keydata.ry = 0;
                        } else {
                            // Ensure neutral position if no input
                            cx.local.keydata.ry = 128;
                        }
                        
                        // Handle X-axis
                        if left_pressed {
                            if right_pressed {
                                cx.local.keydata.rx = 64;
                            } else {
                                cx.local.keydata.rx = 0;
                            }
                        } else if right_pressed {
                            cx.local.keydata.rx = 255;
                        } else {
                            // Ensure neutral position if no input
                            cx.local.keydata.rx = 128;
                        }
                    }
                } else {
                    // Set neutral values for unconfigured analog sticks
                    cx.local.keydata.rx = cx.local.pinout.get_neutral_value(PinType::Rx);
                    cx.local.keydata.ry = cx.local.pinout.get_neutral_value(PinType::Ry);
                }
                
                // If neither analog toggle is active or toggles not configured, process D-Pad
                if (!t_analog_left && !t_analog_right) {
                    // Check up and down, clean SOCD with safe error handling
                    if up_pressed {
                        dpad |= spc::HAT_MASK_UP;
                    }
                    if down_pressed {
                        dpad |= spc::HAT_MASK_DOWN;
                    }
                    // Check left and right
                    if left_pressed {
                        dpad |= spc::HAT_MASK_LEFT;
                    }
                    if right_pressed {
                        dpad |= spc::HAT_MASK_RIGHT;
                    }
                }
                
                // Always update the PadReport regardless of which toggle is active
                // This ensures all button presses are registered
                let mut updated_keys = spc::PadReport::new(&cx.local.keydata);
                
                // Ensure hat switch is properly set if D-pad was active
                if dpad > 0 {
                    updated_keys.set_hat(dpad);
                }
                
                // Advanced debug mode to test multiple button combinations
                // This helps identify which buttons the application recognizes
                let debug_mode = 2; // 0=off, 1=A button, 2=X+Y, 3=all buttons
                
                if debug_mode > 0 {
                    match debug_mode {
                        1 => {
                            // Force A button only
                            cx.local.keydata.buttons |= spc::KEY_MASK_A;
                        },
                        2 => {
                            // Force X+Y buttons (many applications recognize these)
                            cx.local.keydata.buttons |= spc::KEY_MASK_X;
                            cx.local.keydata.buttons |= spc::KEY_MASK_Y;
                        },
                        3 => {
                            // Force all standard buttons
                            cx.local.keydata.buttons = 0xFFFF; // All 16 bits set
                        },
                        _ => {}
                    }
                    
                    // Update with forced buttons
                    updated_keys = spc::PadReport::new(&cx.local.keydata);
                    
                    // Also force D-pad up to test hat switch
                    if debug_mode == 3 {
                        updated_keys.set_hat(spc::HAT_MASK_UP);
                    } else if dpad > 0 {
                        // Otherwise preserve existing hat state
                        updated_keys.set_hat(dpad);
                    }
                }
                
                // Copy the updated values to the shared keys object
                *keys = updated_keys;
                
                // Reset the buttons for the next update, but after we've already sent the report
                cx.local.keydata.buttons = 0;
            });
            
            // Significantly increase the delay to further reduce power consumption
            // This reduces the polling frequency which is a major factor in power usage
            // Adjust this value if responsiveness becomes an issue
            for _ in 0..50000 {
                core::hint::spin_loop();
            }
        }
    }
}
