#![no_std]
#![no_main]

use unflappable::{debouncer_uninit, Debouncer, default::ActiveLow};
//use panic_halt as _;
use riscv_atomic_emulation_trap as _;
mod report;
mod keydata;

// Initialize the debouncer
static DEBOUNCER: Debouncer<PinType, ActiveLow> = debouncer_uninit!();

// Set bounce time in ms
static BOUNCE_TIME: u16 1

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
        mode
    } else {
        changed = false;
        mode
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

fn processSmash(& pins: arduino_hal::gpio::Pins) {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes half values
    if buttons.pressed(debounceshift) {
        if buttons.pressed(debounceup) {
            stickreport.ly = 192;
        } else if buttons.pressed(debouncedown) {
            stickreport.ly = 64;
        }
        if buttons.pressed(debounceleft) {
            stickreport.lx = 64;
        } else if buttons.pressed(debounceright) {
            stickreport.lx = 192;
        }
    } else { // report max values for axies
        if buttons.pressed(debounceup) {
            stickreport.ly = 255;
        } else if buttons.pressed(debouncedown) {
            stickreport.ly = 0;
        }
        if buttons.pressed(debounceleft) {
            stickreport.lx = 0;
        } else if buttons.pressed(debounceright) {
            stickreport.lx = 255;
        }
    }
}

fn processAnalog(& pins: arduino_hal::gpio::Pins) {
    // Analog modes don't change the dpad state
    // Treat the directions as analog input
    // shift makes the input register right stick
    if buttons.pressed(debounceshift) {
        if buttons.pressed(debounceup) {
            stickreport.ry = 255;
        } else if buttons.pressed(debouncedown) {
            stickreport.ry = 0;
        }
        if buttons.pressed(debounceleft) {
            stickreport.rx = 0;
        } else if buttons.pressed(debounceright) {
            stickreport.rx = 255;
        }
    } else {
        if buttons.pressed(debounceup) {
            stickreport.ly = 255;
        } else if buttons.pressed(debouncedown) {
            stickreport.ly = 0;
        }
        if buttons.pressed(debounceleft) {
            stickreport.lx = 0;
        } else if buttons.pressed(debounceright) {
            stickreport.lx = 255;
        }
    }
}

fn processDpad(& pins: arduino_hal::gpio::Pins, stickreport) {
    // Dpad modes don't change the analog state
    // Treat the directions as digital input
    // shift makes the input register SOCD... ish

    // Check first if shift is pressed and switch on that.
    // Shift is meant to provide an input similar to a SOCD controller
    // 
    // Shift first negates left and right when up or down is pressed
    // Next, it negates up if left and right were not present
    // Then it changes Down to UP if present.
    if buttons.pressed(debounceshift) {
        if buttons.pressed(debounceup) {
            if buttons.pressed(debounceleft) {
                stickreport.hat = PAD_MASK_UP;
            } else if buttons.pressed(debounceright) {
                stickreport.hat = PAD_MASK_UP;
            } else {
                stickreport.hat = PAD_MASK_NONE;
            }
        } else if buttons.pressed(debouncedown) {
            if buttons.pressed(debounceleft) {
                stickreport.hat = PAD_MASK_DOWN;
            } else if buttons.pressed(debounceright) {
                stickreport.hat = PAD_MASK_DOWN;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    // Without Shift pressed, the directions are normal
    } else {
        if buttons.pressed(debounceup) {
            if buttons.pressed(debounceleft) {
                stickreport.hat = PAD_MASK_UPLEFT;
            } else if buttons.pressed(debounceright) {
                stickreport.hat = PAD_MASK_UPRIGHT;
            } else {
                stickreport.hat = PAD_MASK_UP;
            }
        } else if buttons.pressed(debouncedown) {
            if buttons.pressed(debounceleft) {
                stickreport.hat = PAD_MASK_DOWNLEFT;
            } else if buttons.pressed debounceright) {
                stickreport.hat = PAD_MASK_DOWNRIGHT;
            } else {
                stickreport.hat = PAD_MASK_DOWN;
            }
        } else if buttons.pressed(debounceleft) {
            stickreport.hat = PAD_MASK_LEFT;
        } else if buttons.pressed(debounceright) {
            stickreport.hat = PAD_MASK_RIGHT;
        } else {
            stickreport.hat = PAD_MASK_NONE;
        }
    }
    return stickreport;
}

fn buttonRead(& pins: arduino_hal::gpio::Pins, mode: InputMode) {
    // Set the report content
    let mut stickreport = keydata {
        buttons = MASK_NONE,
        hat = PAD_MASK_NONE,
        padding = 0,
        lx = 128,
        ly = 128,
        rx = 128,
        ry = 128,
    };

    match mode {
        InputMode::Smash => processSmash(pins, stickreport),
        InputMode::Analog => processAnalog(pins, stickreport),
        InputMode::Dpad => processDpad(pins, stickreport),
    }
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
fn shipit(stickreport: &keydata) {
    // Send the report
    let usb_alloc = UsbBus::new(usb);
    let mut hid = Hid::<PadReport, _>::new(&stickreport);
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
    let mut mode = Dpad;

    loop {
        // poll the debouncer
        unsafe {
            DEBOUNCER.poll()?;
        }
        // Read what is pressed
        let mut buttonstate = buttonRead(debouncebuttons, mut &mode);
        // Update the USB HID report
        shipit(&buttonstate);
    }
}
