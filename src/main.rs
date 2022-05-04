#![no_std]
#![no_main]

use panic_halt as _;
// use usbd_hid_device::HidReport;
// use arduino_hal::port;
use arduino_hal;
// use debouncr::{debounce_8, Debouncer, Edge, Repeat4};
use debouncr::debounce_8;
mod report;
use report::KeyData;
pub mod switches;
use switches::Switch;

// Set bounce time in ms
static BOUNCE_TIME: u16 = 1;

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
enum InputMode {
    Dpad,
    Analog,
    Smash,
}

// Swap Input mode by pressing HOME and SHIFT
fn checkModeChange (
    buttons: &[Switch], 
    mode: &InputMode, 
    _changed: &mut bool, 
    redlight: &arduino_hal::port::Pin<arduino_hal::port::mode::Output>, 
    bluelight: &arduino_hal::port::Pin<arduino_hal::port::mode::Output>
) -> InputMode {
    if !*_changed && buttons[switches::SwitchShift].is_pressed() && buttons[switches::SwitchHome].is_pressed() {
        match mode {
            InputMode::Dpad => {
                mode = &InputMode::Analog;
                redlight.set_high();
                bluelight.set_high();
            },
            InputMode::Analog => {
                mode = &InputMode::Smash;
                redlight.set_high();
                bluelight.set_low();
            },
            InputMode::Smash => {
                mode = &InputMode::Dpad;
                redlight.set_low();
                bluelight.set_high();
            },
        }
        let _changed = true;
        return *mode;
    } else {
        let _changed = false;
        return *mode;
    }
}

fn processSmash(buttons: &[Switch], stickreport: &mut report::KeyData) -> report::KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes half values
    if buttons[switches::SwitchShift].is_pressed() {
        if buttons[switches::SwitchUp].is_pressed() {
            stickreport.ly = 192;
        } else if buttons[switches::SwitchDown].is_pressed() {
            stickreport.ly = 64;
        }
        if buttons[switches::SwitchLeft].is_pressed() {
            stickreport.lx = 64;
        } else if buttons[switches::SwitchRight].is_pressed() {
            stickreport.lx = 192;
        }
    } else { // report max values for axies
        if buttons[switches::SwitchUp].is_pressed() {
            stickreport.ly = 255;
        } else if buttons[switches::SwitchDown].is_pressed() {
            stickreport.ly = 0;
        }
        if buttons[switches::SwitchLeft].is_pressed() {
            stickreport.lx = 0;
        } else if buttons[switches::SwitchRight].is_pressed() {
            stickreport.lx = 255;
        }
    }
    return *stickreport;
}

fn processAnalog(buttons: &[Switch], stickreport: &mut KeyData) -> KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes the input register right stick
    if buttons[switches::SwitchShift].is_pressed() {
        if buttons[switches::SwitchUp].is_pressed() {
            stickreport.ry = 255;
        } else if buttons[switches::SwitchDown].is_pressed() {
            stickreport.ry = 0;
        }
        if buttons[switches::SwitchLeft].is_pressed() {
            stickreport.rx = 0;
        } else if buttons[switches::SwitchRight].is_pressed() {
            stickreport.rx = 255;
        }
    } else {
        if buttons[switches::SwitchUp].is_pressed() {
            stickreport.ly = 255;
        } else if buttons[switches::SwitchDown].is_pressed() {
            stickreport.ly = 0;
        }
        if buttons[switches::SwitchLeft].is_pressed() {
            stickreport.lx = 0;
        } else if buttons[switches::SwitchRight].is_pressed() {
            stickreport.lx = 255;
        }
    }
    return *stickreport;
}

fn processDpad(buttons: &[Switch], stickreport: &mut KeyData) -> KeyData {
    // Dpad modes don't change the analog state
    // Treat the directions as digital input
    // shift makes the input register SOCD... ish

    // Check first if shift is pressed and switch on that.
    // Shift is meant to provide an input similar to a SOCD controller
    // 
    // Shift first negates left and right when up or down is pressed
    // Next, it negates up if left and right were not present
    // Then it changes Down to UP if present.
    if buttons[switches::SwitchShift].is_pressed() {
        if buttons[switches::SwitchUp].is_pressed() {
            if buttons[switches::SwitchLeft].is_pressed() {
                stickreport.hat = PAD_MASK_UP;
            } else if buttons[switches::SwitchRight].is_pressed() {
                stickreport.hat = PAD_MASK_UP;
            } else {
                stickreport.hat = PAD_MASK_NONE;
            }
        } else if buttons[switches::SwitchDown].is_pressed() {
            if buttons[switches::SwitchLeft].is_pressed() {
                stickreport.hat = PAD_MASK_DOWN;
            } else if buttons[switches::SwitchRight].is_pressed() {
                stickreport.hat = PAD_MASK_DOWN;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    // Without Shift pressed, the directions are normal
    } else {
        if buttons[switches::SwitchUp].is_pressed() {
            if buttons[switches::SwitchLeft].is_pressed() {
                stickreport.hat = PAD_MASK_UPLEFT;
            } else if buttons[switches::SwitchRight].is_pressed() {
                stickreport.hat = PAD_MASK_UPRIGHT;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else if buttons[switches::SwitchDown].is_pressed() {
            if buttons[switches::SwitchLeft].is_pressed() {
                stickreport.hat = PAD_MASK_DOWNLEFT;
            } else if buttons[switches::SwitchRight].is_pressed() {
                stickreport.hat = PAD_MASK_DOWNRIGHT;
            } else {
                stickreport.hat = PAD_MASK_DOWN;
            }
        } else if buttons[switches::SwitchLeft].is_pressed() {
            stickreport.hat = PAD_MASK_LEFT;
        } else if buttons[switches::SwitchRight].is_pressed() {
            stickreport.hat = PAD_MASK_RIGHT;
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    }
    return *stickreport;
}

fn buttonRead(signals: &[Switch], mode: InputMode) -> KeyData {
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
        InputMode::Smash => processSmash(signals, &mut stickreport),
        InputMode::Analog => processAnalog(signals, &mut stickreport),
        InputMode::Dpad => processDpad(signals, &mut stickreport),
    };

    // read buttons
    // if button is pressed, set the bit
    if signals[switches::SwitchA].is_high() {
        stickreport.buttons |= MASK_A;
    }
    if signals[switches::SwitchB].is_high() {
        stickreport.buttons |= MASK_B;
    }
    if signals[switches::SwitchX].is_high() {
        stickreport.buttons |= MASK_X;
    }
    if signals[switches::SwitchY].is_high() {
        stickreport.buttons |= MASK_Y;
    }
    if signals[switches::SwitchL1].is_high() {
        stickreport.buttons |= MASK_R1;
    }
    if signals[switches::SwitchR1].is_high() {
        stickreport.buttons |= MASK_R2;
    }
    if signals[switches::SwitchL2].is_high() {
        stickreport.buttons |= MASK_L1;
    }
    if signals[switches::SwitchR2].is_high() {
        stickreport.buttons |= MASK_L2;
    }
    if signals[switches::SwitchSelect].is_high() {
        stickreport.buttons |= MASK_HOME;
    }
    if signals[switches::SwitchStart].is_high() {
        stickreport.buttons |= MASK_SELECT;
    }
    if signals[switches::SwitchHome].is_high() {
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
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // Setup pin modes
    let mut redlight = pins.a3.into_output().downgrade();    // Red LED
    let mut bluelight = pins.d4.into_output().downgrade();   // Blue LED
    let buttona = pins.d3.downgrade();                   // Button A
    let buttonb = pins.a1.downgrade();                   // Button B
    let buttonx = pins.a0.downgrade();                   // Button X
    let buttony = pins.sck.downgrade();                  // Button Y
    let buttonl1 = pins.a1.downgrade();                  // Button L1
    let buttonr1 = pins.d5.downgrade();                  // Button R1
    let buttonl2 = pins.a2.downgrade();                  // Button L2
    let buttonr2 = pins.d0.downgrade();                  // Button R2
    let buttonselect = pins.miso.downgrade();            // Button Select
    let buttonstart = pins.d10.downgrade();              // Button Start
    let buttonhome = pins.mosi.downgrade();              // Button Home
    let buttonshift = pins.d2.downgrade();               // Button Shift
    let buttonup = pins.d7.downgrade();                  // Button Up
    let buttondown = pins.d8.downgrade();                // Button Down
    let buttonleft = pins.d6.downgrade();                // Button Left
    let buttonright = pins.d9.downgrade();               // Button Right

    let pin_array = [
        buttona,
        buttonb,
        buttonx,
        buttony,
        buttonl1,
        buttonr1,
        buttonl2,
        buttonr2,
        buttonselect,
        buttonstart,
        buttonhome,
        buttonshift,
        buttonup,
        buttondown,
        buttonleft,
        buttonright,
    ];

    // Initialize the debouncer
    // I should use this somewhere...
    let _debouncer = debounce_8(true);
    // Package the keys into a struct
    let mut gamepad_signals = switches::build_gamepad(&pin_array);

    // Set the initial state of the LEDs and input mode
    redlight.set_high();
    bluelight.set_high();
    let mode = InputMode::Dpad;
    let mut _changed = false; 
    loop {
        // poll the debouncer
        let gamepad_signals = switches::poll_debouncers(&mut gamepad_signals);
        // Check for mode changes
        let mode = checkModeChange(&gamepad_signals, &mode, &mut _changed, &redlight, &bluelight);
        // Read what is pressed
        let buttonstate = buttonRead(&gamepad_signals, mode);
        // Update the USB HID report
        shipit(&buttonstate);
    }
}
