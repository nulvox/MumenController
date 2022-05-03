#![no_std]
#![no_main]

use unflappable::{debouncer_uninit, Debouncer, default::ActiveLow};
use panic_halt as _;
use usbd_hid_device::{HidReport, HidReportDescriptor};

// use usbd_human_interface_device::prelude::*;
use arduino_hal::port;
mod report;
use report::KeyData;

// Initialize the debouncer
static DEBOUNCER: Debouncer<PinType, ActiveLow> = debouncer_uninit!();

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
fn checkModeChange (buttons: &arduino_hal::port::Pins, mode: &InputMode, changed: &bool, redlight: &arduino_hal::port::Pin<mode::Output>, bluelight: &arduino_hal::port::Pin<mode::Output>) -> InputMode {
    if !changed && &buttons.ButtonSHIFT.pressed() && &buttons.ButtonHOME.pressed() {
        match mode {
            Dpad => {
                mode = &InputMode::Analog;
                *redlight.set_high();
                *bluelight.set_high();
            },
            Analog => {
                mode = &InputMode::Smash;
                *redlight.set_high();
                *bluelight.set_low();
            },
            Smash => {
                mode = &InputMode::Dpad;
                *redlight.set_low();
                *bluelight.set_high();
            },
        }
        let changed = true;
        return *mode;
    } else {
        let changed = false;
        return *mode;
    }
}

fn processSmash(buttons: &arduino_hal::port::Pins, stickreport: &KeyData) -> &KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes half values
    if buttons.pressed(buttons.debounceshift) {
        if buttons.pressed(buttons.debounceup) {
            stickreport.ly = 192;
        } else if buttons.pressed(buttons.debouncedown) {
            stickreport.ly = 64;
        }
        if buttons.pressed(buttons.debounceleft) {
            stickreport.lx = 64;
        } else if buttons.pressed(buttons.debounceright) {
            stickreport.lx = 192;
        }
    } else { // report max values for axies
        if buttons.pressed(buttons.debounceup) {
            stickreport.ly = 255;
        } else if buttons.pressed(buttons.debouncedown) {
            stickreport.ly = 0;
        }
        if buttons.pressed(buttons.debounceleft) {
            stickreport.lx = 0;
        } else if buttons.pressed(buttons.debounceright) {
            stickreport.lx = 255;
        }
    }
    return stickreport;
}

fn processAnalog(buttons: &arduino_hal::port::Pins, stickreport: &KeyData) -> &KeyData {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes the input register right stick
    if buttons.pressed(buttons.debounceshift) {
        if buttons.pressed(buttons.debounceup) {
            stickreport.ry = 255;
        } else if buttons.pressed(buttons.debouncedown) {
            stickreport.ry = 0;
        }
        if buttons.pressed(buttons.debounceleft) {
            stickreport.rx = 0;
        } else if buttons.pressed(buttons.debounceright) {
            stickreport.rx = 255;
        }
    } else {
        if buttons.pressed(buttons.debounceup) {
            stickreport.ly = 255;
        } else if buttons.pressed(buttons.debouncedown) {
            stickreport.ly = 0;
        }
        if buttons.pressed(buttons.debounceleft) {
            stickreport.lx = 0;
        } else if buttons.pressed(buttons.debounceright) {
            stickreport.lx = 255;
        }
    }
    return stickreport;
}

fn processDpad(buttons: &arduino_hal::port::Pins, stickreport: &KeyData) -> &KeyData {
    // Dpad modes don't change the analog state
    // Treat the directions as digital input
    // shift makes the input register SOCD... ish

    // Check first if shift is pressed and switch on that.
    // Shift is meant to provide an input similar to a SOCD controller
    // 
    // Shift first negates left and right when up or down is pressed
    // Next, it negates up if left and right were not present
    // Then it changes Down to UP if present.
    if buttons.pressed(buttons.debounceshift) {
        if buttons.pressed(buttons.debounceup) {
            if buttons.pressed(buttons.debounceleft) {
                stickreport.hat = PAD_MASK_UP;
            } else if buttons.pressed(buttons.debounceright) {
                stickreport.hat = PAD_MASK_UP;
            } else {
                stickreport.hat = PAD_MASK_NONE;
            }
        } else if buttons.pressed(buttons.debouncedown) {
            if buttons.pressed(buttons.debounceleft) {
                stickreport.hat = PAD_MASK_DOWN;
            } else if buttons.pressed(buttons.debounceright) {
                stickreport.hat = PAD_MASK_DOWN;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    // Without Shift pressed, the directions are normal
    } else {
        if buttons.pressed(buttons.debounceup) {
            if buttons.pressed(buttons.debounceleft) {
                stickreport.hat = PAD_MASK_UPLEFT;
            } else if buttons.pressed(buttons.debounceright) {
                stickreport.hat = PAD_MASK_UPRIGHT;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else if buttons.pressed(buttons.debouncedown) {
            if buttons.pressed(buttons.debounceleft) {
                stickreport.hat = PAD_MASK_DOWNLEFT;
            } else if buttons.pressed(buttons.debounceright) {
                stickreport.hat = PAD_MASK_DOWNRIGHT;
            } else {
                stickreport.hat = PAD_MASK_DOWN;
            }
        } else if buttons.pressed(buttons.debounceleft) {
            stickreport.hat = PAD_MASK_LEFT;
        } else if buttons.pressed(buttons.debounceright) {
            stickreport.hat = PAD_MASK_RIGHT;
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    }
    return stickreport;
}

fn buttonRead(pins: &arduino_hal::port::Pins, mode: InputMode) -> KeyData {
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
        InputMode::Smash => processSmash(pins, &stickreport),
        InputMode::Analog => processAnalog(pins, &stickreport),
        InputMode::Dpad => processDpad(pins, &stickreport),
    };

    // read buttons
    // if button is pressed, set the bit
    if pins.a3.is_high() {
        stickreport.buttons |= MASK_A;
    }
    if pins.a1.is_high() {
        stickreport.buttons |= MASK_B;
    }
    if pins.a0.is_high() {
        stickreport.buttons |= MASK_X;
    }
    if pins.sck.is_high() {
        stickreport.buttons |= MASK_Y;
    }
    if pins.d5.is_high() {
        stickreport.buttons |= MASK_R1;
    }
    if pins.d0.is_high() {
        stickreport.buttons |= MASK_R2;
    }
    if pins.a1.is_high() {
        stickreport.buttons |= MASK_L1;
    }
    if pins.a2.is_high() {
        stickreport.buttons |= MASK_L2;
    }
    if pins.mosi.is_high() {
        stickreport.buttons |= MASK_HOME;
    }
    if pins.miso.is_high() {
        stickreport.buttons |= MASK_SELECT;
    }
    if pins.d10.is_high() {
        stickreport.buttons |= MASK_START;
    }
    return stickreport;
}

// Build the actual HID Report and send it over the wire
fn shipit(stickreport: &KeyData) {
    // Send the report
    // let usb_alloc = UsbBus::new(usb);
    // let mut hid = Hid::<PadReport, _>::new(&stickreport);
    let mut hid = usbd_hid_device::Hid::<HidReport, _>::new(&stickreport);
    hid.send(&hid).unwrap();
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    
    // let pinnames = definePins(&pins);

    // Setup pin modes
    let mut redlight = pins.a3.into_output().downgrade();
    let mut bluelight = pins.d4.into_output().downgrade();
    let mut buttona = pins.d3.downgrade();
    let mut buttonb = pins.a1.downgrade();
    let mut buttonx = pins.a0.downgrade();
    let mut buttony = pins.sck.downgrade();
    let mut buttonl1 = pins.a1.downgrade();
    let mut buttonr1 = pins.d5.downgrade();
    let mut buttonl2 = pins.a2.downgrade();
    let mut buttonr2 = pins.d0.downgrade();
    let mut buttonselect = pins.miso.downgrade();
    let mut buttonstart = pins.d10.downgrade();
    let mut buttonup = pins.d7.downgrade();
    let mut buttondown = pins.d8.downgrade();
    let mut buttonleft = pins.d6.downgrade();
    let mut buttonright = pins.d9.downgrade();
    let mut buttonshift = pins.d2.downgrade();
    let mut buttonhome = pins.mosi.downgrade();

    // Setup debounce enum
    // Handle these errors now...
    unsafe {
        DEBOUNCER.init(buttona);
        DEBOUNCER.init(buttonb);
        DEBOUNCER.init(buttonx);
        DEBOUNCER.init(buttony);
        DEBOUNCER.init(buttonl1);
        DEBOUNCER.init(buttonr1);
        DEBOUNCER.init(buttonl2);
        DEBOUNCER.init(buttonr2);
        DEBOUNCER.init(buttonselect);
        DEBOUNCER.init(buttonstart);
        DEBOUNCER.init(buttonup);
        DEBOUNCER.init(buttondown);
        DEBOUNCER.init(buttonleft);
        DEBOUNCER.init(buttonright);
        DEBOUNCER.init(buttonshift);
        DEBOUNCER.init(buttonhome);
    };

    // Set the initial state of the LEDs and input mode
    redlight.set_high();
    bluelight.set_high();
    let mut mode = InputMode::Dpad;
    let mut changed = false; 
    loop {
        // poll the debouncer
        unsafe {
            DEBOUNCER.poll()?;
        }
        // Read what is pressed
        let mut buttonstate = buttonRead(&pins, mode);
        // Check for mode changes
        let mode = checkModeChange(&pins, &mode, &changed, &redlight, &bluelight);
        // Update the USB HID report
        shipit(&buttonstate);
    }
}
