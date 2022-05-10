#![no_std]
#![no_main]

use panic_halt as _;
use arduino_hal;
mod report;
use report::KeyData;
pub mod switches;
use switches::Switch;

// Button state masks
static MASK_A: u16 = 0x0004;
static MASK_B: u16 = 0x0002;
static MASK_X: u16 = 0x0008;
static MASK_Y: u16 = 0x0001;
static MASK_L1: u16 = 0x0010;
static MASK_R1: u16 = 0x0020;
static MASK_L2: u16 = 0x0040;
static MASK_R2: u16 = 0x0080;
static MASK_SELECT: u16 = 0x0100;
static MASK_START: u16 = 0x0200;
static MASK_HOME: u16 = 0x1000;
static MASK_NONE: u16 = 0x0000;

// Dpad Hat switch state masks
static PAD_MASK_UP: u8 = 0x00;
static PAD_MASK_UPRIGHT: u8 = 0x01;
static PAD_MASK_RIGHT: u8 = 0x02;
static PAD_MASK_DOWNRIGHT: u8 = 0x03;
static PAD_MASK_DOWN: u8 = 0x04;
static PAD_MASK_DOWNLEFT: u8 = 0x05;
static PAD_MASK_LEFT: u8 = 0x06;
static PAD_MASK_UPLEFT: u8 = 0x07;
static PAD_MASK_NONE: u8 = 0x08;

// Mode Selection
#[derive(Debug, Copy, Clone)]
enum InputMode {
    Dpad,
    Analog,
    Smash,
}

// Swap Input mode by pressing HOME and SHIFT
fn process_mode_change (
    buttons: &[Switch], 
    mut mode: InputMode, 
    _changed: &mut bool, 
    indicators: &mut [arduino_hal::port::Pin<arduino_hal::port::mode::Output>; 2]
) -> InputMode {
    if !*_changed && buttons[switches::SWITCH_SHIFT].is_pressed() && buttons[switches::SWITCH_HOME].is_pressed() {
        match mode {
            InputMode::Dpad => {
                mode = InputMode::Analog;
                indicators[0].set_high(); // Turn on Red LED
                indicators[1].set_high(); // Turn on Blue LED
            },
            InputMode::Analog => {
                mode = InputMode::Smash;
                indicators[0].set_high(); // Turn on Red LED
                indicators[1].set_low();  // Turn off Blue LED
            },
            InputMode::Smash => {
                mode = InputMode::Dpad;
                indicators[0].set_low();  // Turn off Red LED
                indicators[1].set_high(); // Turn on Blue LED
            },
        }
        let _changed = true;
        return mode;
    } else {
        let _changed = false;
        return mode;
    }
}

fn process_smash(buttons: &[Switch], stickreport: &mut report::KeyData) -> report::KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes half values
    if buttons[switches::SWITCH_SHIFT].is_pressed() {
        if buttons[switches::SWITCH_UP].is_pressed() {
            stickreport.ly = 192;
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            stickreport.ly = 64;
        }
        if buttons[switches::SWITCH_LEFT].is_pressed() {
            stickreport.lx = 64;
        } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
            stickreport.lx = 192;
        }
    } else { // report max values for axies
        if buttons[switches::SWITCH_UP].is_pressed() {
            stickreport.ly = 255;
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            stickreport.ly = 0;
        }
        if buttons[switches::SWITCH_LEFT].is_pressed() {
            stickreport.lx = 0;
        } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
            stickreport.lx = 255;
        }
    }
    return *stickreport;
}

fn process_analog(buttons: &[Switch], stickreport: &mut KeyData) -> KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes the input register right stick
    if buttons[switches::SWITCH_SHIFT].is_pressed() {
        if buttons[switches::SWITCH_UP].is_pressed() {
            stickreport.ry = 255;
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            stickreport.ry = 0;
        }
        if buttons[switches::SWITCH_LEFT].is_pressed() {
            stickreport.rx = 0;
        } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
            stickreport.rx = 255;
        }
    } else {
        if buttons[switches::SWITCH_UP].is_pressed() {
            stickreport.ly = 255;
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            stickreport.ly = 0;
        }
        if buttons[switches::SWITCH_LEFT].is_pressed() {
            stickreport.lx = 0;
        } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
            stickreport.lx = 255;
        }
    }
    return *stickreport;
}

fn process_dpad(buttons: &[Switch], stickreport: &mut KeyData) -> KeyData {
    // Dpad modes don't change the analog state
    // Treat the directions as digital input
    // shift makes the input register SOCD... ish

    // Check first if shift is pressed and switch on that.
    // Shift is meant to provide an input similar to a SOCD controller
    // 
    // Shift first negates left and right when up or down is pressed
    // Next, it negates up if left and right were not present
    // Then it changes Down to UP if present.
    if buttons[switches::SWITCH_SHIFT].is_pressed() {
        if buttons[switches::SWITCH_UP].is_pressed() {
            if buttons[switches::SWITCH_LEFT].is_pressed() {
                stickreport.hat = PAD_MASK_UP;
            } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
                stickreport.hat = PAD_MASK_UP;
            } else {
                stickreport.hat = PAD_MASK_NONE;
            }
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            if buttons[switches::SWITCH_LEFT].is_pressed() {
                stickreport.hat = PAD_MASK_DOWN;
            } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
                stickreport.hat = PAD_MASK_DOWN;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    // Without Shift pressed, the directions are normal
    } else {
        if buttons[switches::SWITCH_UP].is_pressed() {
            if buttons[switches::SWITCH_LEFT].is_pressed() {
                stickreport.hat = PAD_MASK_UPLEFT;
            } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
                stickreport.hat = PAD_MASK_UPRIGHT;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else if buttons[switches::SWITCH_DOWN].is_pressed() {
            if buttons[switches::SWITCH_LEFT].is_pressed() {
                stickreport.hat = PAD_MASK_DOWNLEFT;
            } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
                stickreport.hat = PAD_MASK_DOWNRIGHT;
            } else {
                stickreport.hat = PAD_MASK_DOWN;
            }
        } else if buttons[switches::SWITCH_LEFT].is_pressed() {
            stickreport.hat = PAD_MASK_LEFT;
        } else if buttons[switches::SWITCH_RIGHT].is_pressed() {
            stickreport.hat = PAD_MASK_RIGHT;
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    }
    return *stickreport;
}

fn button_read(signals: &[Switch], mode: InputMode) -> KeyData {
    // Set the report content
    let mut stickreport = KeyData {
        buttons: MASK_NONE,
        hat: PAD_MASK_NONE,
        padding: 0,
        lx: 128,
        ly: 128,
        rx: 128,
        ry: 128,
    };

    match mode {
        InputMode::Smash => process_smash(signals, &mut stickreport),
        InputMode::Analog => process_analog(signals, &mut stickreport),
        InputMode::Dpad => process_dpad(signals, &mut stickreport),
    };

    // read buttons
    // if button is pressed, set the bit
    if signals[switches::SWITCH_A].is_high() {
        stickreport.buttons |= MASK_A;
    }
    if signals[switches::SWITCH_B].is_high() {
        stickreport.buttons |= MASK_B;
    }
    if signals[switches::SWITCH_X].is_high() {
        stickreport.buttons |= MASK_X;
    }
    if signals[switches::SWITCH_Y].is_high() {
        stickreport.buttons |= MASK_Y;
    }
    if signals[switches::SWITCH_L1].is_high() {
        stickreport.buttons |= MASK_R1;
    }
    if signals[switches::SWITCH_R1].is_high() {
        stickreport.buttons |= MASK_R2;
    }
    if signals[switches::SWITCH_L2].is_high() {
        stickreport.buttons |= MASK_L1;
    }
    if signals[switches::SWITCH_R2].is_high() {
        stickreport.buttons |= MASK_L2;
    }
    if signals[switches::SWITCH_SELECT].is_high() {
        stickreport.buttons |= MASK_HOME;
    }
    if signals[switches::SWITCH_START].is_high() {
        stickreport.buttons |= MASK_SELECT;
    }
    if signals[switches::SWITCH_HOME].is_high() {
        stickreport.buttons |= MASK_START;
    }
    return stickreport;
}

// Build the actual HID Report and send it over the wire
fn shipit(stickreport: &report::KeyData) {
    // Send the report

    // this stuff might be important... check it out
    // let usb_alloc = UsbBus::new(usb);
    // let mut hid = Hid::<PadReport, _>::new(&stickreport);
    let hid = report::PadReport::new(&stickreport);
    hid.send();
}

#[arduino_hal::entry]
fn main() -> ! {
    // Package the keys into a struct
    let mut gamepad_signals = switches::build_gamepad();
    let mut indicators = switches::build_indicators();

    // Set the initial state of the LEDs and input mode
    indicators[0].set_high(); // Turn on the Red LED
    indicators[1].set_high(); // Turn on the Blue LED
    let _mode = InputMode::Dpad;
    let mut _changed = false; 

    // Set up the USB interface
        //USB
        let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
            pac.USBCTRL_REGS,
            pac.USBCTRL_DPRAM,
            clocks.usb_clock,
            true,
            &mut pac.RESETS,
        ));
    
        let mut fightstick = UsbHidClassBuilder::new()
            .add_interface(
                usbd_human_interface_device::device::switch_gamepad::SwitchGamepadInterface::default_config(),
            )
            .build(&usb_bus);
    
        //https://pid.codes
        let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
            .manufacturer("Me... I made it...")
            .product("Mumen Controller")
            .serial_number("breakfast5")
            .supports_remote_wakeup(false)
            .build();

    loop {
        // poll the debouncer
        let gamepad_signals = switches::poll_debouncers(&mut gamepad_signals);
        // Scope the borrow of gamepad signals
        {
            // Check for mode changes
            let _mode = process_mode_change(gamepad_signals, _mode, &mut _changed, &mut indicators);
        }
        // Read what is pressed
        let buttonstate = button_read(gamepad_signals, _mode);
        // Update the USB HID report
        // shipit(&buttonstate);
        fightstick.interface().write_report(&buttonstate);
    }
}
