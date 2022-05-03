#![no_std]
#![no_main]

use unflappable::{debouncer_uninit, Debouncer, default::ActiveLow};
use panic_halt as _;
use usbd_hid_device::{HidReport, HidReportDescriptor};

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

struct KeyData {
    buttons: u16,
    hat: u8,
    padding: u8,
    lx: u8,
    ly: u8,
    rx: u8,
    ry: u8,
}

/// Hid report for a 3-button mouse with a wheel.
struct UsbReport {
    // Bytes usage:
    // byte 0..1: bits 0..13 = buttons, 14 and 15 are unused at this time
    // byte 2: dpad hat switch
    // byte 3: padding for hat switch
    // byte 4: L stick X
    // byte 5: L stick Y
    // byte 6: R stick X
    // byte 7: R stick Y
    bytes: [u8; 8],
}

impl PadReport {
    pub fn new(btnstate: &enum) -> Self {
        let btnhigh: u8 = btnstate.buttons >> 8;
        let btnlow: u8 = btnstate.buttons & 0xFF;
        PadReport { 
            bytes: [ 
                btnhigh, 
                btnlow, 
                btnstate.hat, 
                0x00, // padding for hat switch
                btnstate.lx, 
                btnstate.ly, 
                btnstate.rx, 
                btnstate.ry, 
            ],
        }
    }
}

impl AsRef<[u8]> for PadReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl HidReport for PadReport {
    const DESCRIPTOR: &'static [u8] = &[
        0x08, 0x01,                   // USAGE_PAGE Generic Desktop
        0x08, 0x05,                   // USAGE Joystick
        0x08, 0x01,                   // COLLECTION Application
            0x08, 0x00,               // Logical Min
            0x08, 0x01,               // Logical Max
            0x08, 0x00,               // Physical Min
            0x08, 0x01,               // Physical Max
            0x08, 0x01,               // REPORT_SIZE 1
            0x08, 0x10,              // REPORT_COUNT 16
            0x08, 0x09,               // USAGE PAGE
            0x08, 0x01,               // USAGE Min
            0x08, 0x10,              // USAGE Max
            0x08, 0x02,               // INPUT
            // Hat switch, 1 nibble with a spare nibble
            0x08, 0x01,               // USAGE Page
            0x08, 0x07,               // LOGICAL Max
            0x10, 0x01, 0x3B,            // PHYSICAL Max
            0x08, 0x04,               // REPORT_SIZE
            0x08, 0x01,               // REPORT_COUNT
            0x08, 0x14,              // UNIT
            0x08, 0x39,              // USAGE
            0x08, 0x42,              // INPUT
            // this is where the spare nibble goes
            0x08, 0x00,               // UNIT
            0x08, 0x01,               // REPORT_COUNT
            0x08, 0x01,               // INPUT
            0x10, 0xFF, 0xFF,            // LOGICAL Max
            0x10, 255,            // PHYSICAL Max
            0x08, 0x08,              // USAGE
            0x08, 0x31,              // USAGE
            0x08, 0x32,              // USAGE
            0x08, 0x35,              // USAGE
            0x08, 0x08,               // REPORT SIZE
            0x08, 0x04,               // REPORT COUNT
            0x08, 0x02,               // INPUT
            // vendor specific byte
            0x10, 0xFF, 0x00,        // USAGE PAGE  (16-bit value, this hack is ugly)
            0x08, 0x20,              // USAGE
            0x08, 0x01,               // REPORT COUNT
            0x08, 0x02,               // INPUT
            // Output, 8 bytes
            0x10, 0x26, 0x21,     // USAGE  (16-bit value, this hack is ugly)
            0x08, 0x08,               // REPORT COUNT
            0x08, 0x02,               // OUTPUT
        0x00 // END COLLECTION
    ];
}

// Mode Selection
enum InputMode {
    Dpad,
    Analog,
    Smash,
}

// Swap Input mode by pressing HOME and SHIFT
fn checkModeChange (buttons, mode: &InputMode, changed: &bool, indicators: &enum) -> &InputMode {
    if !changed && buttons.ButtonSHIFT.pressed() && buttons.ButtonHOME.pressed() {
        match mode {
            Dpad => {
                mode = &InputMode::Analog;
                indicators.redlight.set_high();
                indicators.bluelight.set_high();
            },
            Analog => {
                mode = &InputMode::Smash;
                indicators.redlight.set_high();
                indicators.bluelight.set_low();
            },
            Smash => {
                mode = &InputMode::Dpad;
                indicators.redlight.set_low();
                indicators.bluelight.set_high();
            },
        }
        let changed = true;
        return mode;
    } else {
        let changed = false;
        return mode;
    }
}

// Define your pinout here
fn definePins(&mut pins: arduino_hal::gpio::Pins) {
    // Return a struct with named objects
    enum {
        RedLight: pins.a3,
        BlueLight: pins.d4,
        ButtonA: pins.d3,
        ButtonB: pins.a1,
        ButtonX: pins.a0,
        ButtonY: pins.d15,
        ButtonL1: pins.d19,
        ButtonR1: pins.d5,
        ButtonL2: pins.a2,
        ButtonR2: pins.d0,
        ButtonSELECT: pins.d14,
        ButtonSTART: pins.d10,
        ButtonUP: pins.d7,
        ButtonDOWN: pins.d8,
        ButtonLEFT: pins.d6,
        ButtonRIGHT: pins.d9,
        ButtonSHIFT: pins.d2,
        ButtonHOME: pins.d16,
    }
}

fn processSmash(& buttons: arduino_hal::gpio::Pins, stickreport: &KeyData) -> &KeyData {
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

fn processAnalog(& buttons: arduino_hal::gpio::Pins, stickreport: &KeyData) -> &KeyData {
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

fn processDpad(& buttons: arduino_hal::gpio::Pins, stickreport: &KeyData) -> &KeyData {
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

fn buttonRead(& pins: arduino_hal::gpio::Pins, mode: InputMode) -> KeyData {
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
    if pins.debouncea.is_high() {
        stickreport.buttons |= MASK_A;
    }
    if pins.debounceb.is_high() {
        stickreport.buttons |= MASK_B;
    }
    if pins.debouncex.is_high() {
        stickreport.buttons |= MASK_X;
    }
    if pins.debouncey.is_high() {
        stickreport.buttons |= MASK_Y;
    }
    if pins.deboucer1.is_high() {
        stickreport.buttons |= MASK_R1;
    }
    if pins.debouncer2.is_high() {
        stickreport.buttons |= MASK_R2;
    }
    if pins.deboucel1.is_high() {
        stickreport.buttons |= MASK_L1;
    }
    if pins.deboucel2.is_high() {
        stickreport.buttons |= MASK_L2;
    }
    if pins.debouncehome.is_high() {
        stickreport.buttons |= MASK_HOME;
    }
    if pins.debounceselect.is_high() {
        stickreport.buttons |= MASK_SELECT;
    }
    if pins.debouncestart.is_high() {
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
    
    let enum pinnames = definePins(mut &pins);

    // Setup pin modes
    let mut redlight = pinnames.RedLight.into_output();
    let mut bluelight = pinnames.BlueLight.into_output();
    let mut buttona = pinnames.ButtonA.into_float();
    let mut buttonb = pinnames.ButtonB.into_float();
    let mut buttonx = pinnames.ButtonX.into_float();
    let mut buttony = pinnames.ButtonY.into_float();
    let mut buttonl1 = pinnames.ButtonL1.into_float();
    let mut buttonr1 = pinnames.ButtonR1.into_float();
    let mut buttonl2 = pinnames.ButtonL2.into_float();
    let mut buttonr2 = pinnames.ButtonR2.into_float();
    let mut buttonselect = pinnames.ButtonSELECT.into_float();
    let mut buttonstart = pinnames.ButtonSTART.into_float();
    let mut buttonup = pinnames.ButtonUP.into_float();
    let mut buttondown = pinnames.ButtonDOWN.into_float();
    let mut buttonleft = pinnames.ButtonLEFT.into_float();
    let mut buttonright = pinnames.ButtonRIGHT.into_float();
    let mut buttonshift = pinnames.ButtonSHIFT.into_float();
    let mut buttonhome = pinnames.ButtonHOME.into_float();

    // Setup debounce enum
    let mut debouncebuttons = enum {
        debouncea = unsafe { DEBOUNCER.init(buttona) }?;
        debounceb = unsafe { DEBOUNCER.init(buttonb) }?;
        debouncex = unsafe { DEBOUNCER.init(buttonx) }?;
        debouncey = unsafe { DEBOUNCER.init(buttony) }?;
        debouncel1 = unsafe { DEBOUNCER.init(buttonl1) }?;
        debouncer1 = unsafe { DEBOUNCER.init(buttonr1) }?;
        debouncel2 = unsafe { DEBOUNCER.init(buttonl2) }?;
        debouncer2 = unsafe { DEBOUNCER.init(buttonr2) }?;
        debounceselect = unsafe { DEBOUNCER.init(buttonselect) }?;
        debouncestart = unsafe { DEBOUNCER.init(buttonstart) }?;
        debounceup = unsafe { DEBOUNCER.init(buttonup) }?;
        debouncedown = unsafe { DEBOUNCER.init(buttondown) }?;
        debounceleft = unsafe { DEBOUNCER.init(buttonleft) }?;
        debounceright = unsafe { DEBOUNCER.init(buttonright) }?;
        debounceshift = unsafe { DEBOUNCER.init(buttonshift) }?;
        debouncehome = unsafe { DEBOUNCER.init(buttonhome) }?;
    }

    // Set the initial state of the LEDs and input mode
    redlight.set_high();
    bluelight.set_high();
    let mut indicators = {redligh, bluelight};
    let mut mode = Dpad;

    loop {
        // poll the debouncer
        unsafe {
            DEBOUNCER.poll()?;
        }
        // Read what is pressed
        let mut buttonstate = buttonRead(debouncebuttons, mut &mode, &indicators);
        // Check for mode changes
        let mode = checkMode(debouncebuttons, mut &mode);
        // Update the USB HID report
        shipit(&buttonstate);
    }
}
