#![no_std]
#![no_main]

use panic_halt as _;

// Button and light pinout
static REDLIGHT: u16 13
static BLUELIGHT: u16 4 
static BUTTONA: u16 3
static BUTTONB: u16 19
static BUTTONX: u16 18
static BUTTONY: u16 15
static BUTTONL1: u16 19
static BUTTONR1: u16 5
static BUTTONL2: u16 20
static BUTTONR2: u16 0
static BUTTONSELECT: u16 14
static BUTTONSTART: u16 10
static BUTTONUP: u16 7
static BUTTONDOWN: u16 8
static BUTTONLEFT: u16 6
static BUTTONRIGHT: u16 9
static BUTTONSHIFT: u16 2
static BUTTONHOME: u16 16

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
static MASK_UP: u16 0x0000
static MASK_UPRIGHT: u16 0x0001
static MASK_RIGHT: u16 0x0002
static MASK_DOWNRIGHT: u16 0x0003
static MASK_DOWN: u16 0x0004
static MASK_DOWNLEFT: u16 0x0005
static MASK_LEFT: u16 0x0006
static MASK_UPLEFT: u16 0x0007
static MASK_NONE: u16 0x0008


#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

    let mut led = pins.d13.into_output();

    loop {
        led.toggle();
        arduino_hal::delay_ms(1000);
    }
}
