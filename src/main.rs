#![no_std]
#![no_main]

use panic_halt as _;

// Button state masks
static MASK_A: u16 0x0004
static MASK_B: u16 0x0002
static MASK_X: u16 0x0008
static MASK_Y: u16 0x0001
static MASK_L1: u16 0x0010
static MASK_R1: u16 0x0020
static MASK_L2: u16 0x0040
static MASK_R2: u16 0x0080
static MASK_SELECT: u16 0x0100
static MASK_START: u16 0x0200
static MASK_HOME: u16 0x1000

// Dpad Hat switch state masks
static PAD_MASK_UP: u16 0x0000
static PAD_MASK_UPRIGHT: u16 0x0001
static PAD_MASK_RIGHT: u16 0x0002
static PAD_MASK_DOWNRIGHT: u16 0x0003
static PAD_MASK_DOWN: u16 0x0004
static PAD_MASK_DOWNLEFT: u16 0x0005
static PAD_MASK_LEFT: u16 0x0006
static PAD_MASK_UPLEFT: u16 0x0007
static PAD_MASK_NONE: u16 0x0008

// Mode Selection
enum InputMode {
    Dpad,
    Analog,
    Smash,
}

// Swap Input mode by pressing HOME and SHIFT
fn checkModeChange (&mut mode: InputMode, &mut changed: bool) {
    if ( !changed && BUTTONSHIFT.pressed() && BUTTONHOME.pressed() ){
        match mode {
            Dpad => {
                mode = Analog;
                redlight.set_high();
                bluelight.set_high();
            },
            Analog => {
                mode = Smash;
                redlight.set_high();
                bluelight.set_low();
            },
            Smash => {
                mode = Dpad;
                redlight.set_low();
                bluelight.set_high();
            },
        }
        let changed = true;
    } else {
        changed = false;
    }
}

fn definePins(&mut pins: arduino_hal::gpio::Pins) {
    // Return a struct with named objects
    return enum {
        REDLIGHT: pins.a3,
        BLUELIGHT: pins.d4,
        BUTTONA: pins.d3,
        BUTTONB: pins.a1,
        BUTTONX: pins.a0,
        BUTTONY: pins.d15,
        BUTTONL1: pins.d19,
        BUTTONR1: pins.d5,
        BUTTONL2: pins.a2,
        BUTTONR2: pins.d0,
        BUTTONSELECT: pins.d14,
        BUTTONSTART: pins.d10,
        BUTTONUP: pins.d7,
        BUTTONDOWN: pins.d8,
        BUTTONLEFT: pins.d6,
        BUTTONRIGHT: pins.d9,
        BUTTONSHIFT: pins.d2,
        BUTTONHOME: pins.d16,
    }
}

fn processSmash(& pins: arduino_hal::gpio::Pins) {
    let mut report = PAD_MASK_NONE;
    // Reat the directions as analog input
    // shift makes half values
}

fn processAnalog(& pins: arduino_hal::gpio::Pins) {
    let mut report = PAD_MASK_NONE;
    // Reat the directions as analog input
    // shift makes the input register right stick
}

fn processDpad(& pins: arduino_hal::gpio::Pins) {
    let mut report = PAD_MASK_NONE;
    // Reat the directions as digital input
    // shift makes the input register SOCD... ish
}

fn processButtons(& pins: arduino_hal::gpio::Pins) {
    // learn to read buttons... for all of this

}

fn buttonRead(& pins: arduino_hal::gpio::Pins) {
    // read buttons
    // if button is pressed, set the bit
    // if button is released, clear the bit
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    
    let enum pinnames = definePins(mut &pins).unwrap();

    // Setup pin modes
    let mut redlight = pinnames.REDLIGHT.into_output();
    let mut bluelight = pinnames.BLUELIGHT.into_output();
    let mut buttona = pinnames.BUTTONA.into_float();
    let mut buttonb = pinnames.BUTTONB.into_float();
    let mut buttonx = pinnames.BUTTONX.into_float();
    let mut buttony = pinnames.BUTTONY.into_float();
    let mut buttonl1 = pinnames.BUTTONL1.into_float();
    let mut buttonr1 = pinnames.BUTTONR1.into_float();
    let mut buttonl2 = pinnames.BUTTONL2.into_float();
    let mut buttonr2 = pinnames.BUTTONR2.into_float();
    let mut buttonselect = pinnames.BUTTONSELECT.into_float();
    let mut buttonstart = pinnames.BUTTONSTART.into_float();
    let mut buttonup = pinnames.BUTTONUP.into_float();
    let mut buttondown = pinnames.BUTTONDOWN.into_float();
    let mut buttonleft = pinnames.BUTTONLEFT.into_float();
    let mut buttonright = pinnames.BUTTONRIGHT.into_float();
    let mut buttonshift = pinnames.BUTTONSHIFT.into_float();
    let mut buttonhome = pinnames.BUTTONHOME.into_float();

    // Setup debounce
    let mut debouncea = Debounce::new(buttona, 1);
    let mut debounceb = Debounce::new(buttonb, 1);
    let mut debouncex = Debounce::new(buttonx, 1);
    let mut debouncey = Debounce::new(buttony, 1);
    let mut debouncel1 = Debounce::new(buttonl1, 1);
    let mut debouncer1 = Debounce::new(buttonr1, 1);
    let mut debouncel2 = Debounce::new(buttonl2, 1);
    let mut debouncer2 = Debounce::new(buttonr2, 1);
    let mut debounceselect = Debounce::new(buttonselect, 1);
    let mut debouncestart = Debounce::new(buttonstart, 1);
    let mut debounceup = Debounce::new(buttonup, 1);
    let mut debouncedown = Debounce::new(buttondown, 1);
    let mut debounceleft = Debounce::new(buttonleft, 1);
    let mut debounceright = Debounce::new(buttonright, 1);
    let mut debouncehome = Debounce::new(buttonhome, 1);
    let mut debounceshift = Debounce::new(buttonshift, 1);

    // Set the initial state of the LEDs and input mode
    redlight.set_high();
    bluelight.set_high();
    let mut mode = Dpad;

    loop {
        buttonRead();
        checkModeChange();
        processButtons();
        HID_Task();
        // Update the USB HID report
        USB_USBTask();
    }
}
