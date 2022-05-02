#![no_std]
#![no_main]

// Button state masks
pub static MASK_A: u16 0x0004
pub static MASK_B: u16 0x0002
pub static MASK_X: u16 0x0008
pub static MASK_Y: u16 0x0001
pub static MASK_L1: u16 0x0010
pub static MASK_R1: u16 0x0020
pub static MASK_L2: u16 0x0040
pub static MASK_R2: u16 0x0080
pub static MASK_SELECT: u16 0x0100
pub static MASK_START: u16 0x0200
pub static MASK_HOME: u16 0x1000
pub static MASK_NONE: u16 0x0000

// Dpad Hat switch state masks
pub static PAD_MASK_UP: u16 0x0000
pub static PAD_MASK_UPRIGHT: u16 0x0001
pub static PAD_MASK_RIGHT: u16 0x0002
pub static PAD_MASK_DOWNRIGHT: u16 0x0003
pub static PAD_MASK_DOWN: u16 0x0004
pub static PAD_MASK_DOWNLEFT: u16 0x0005
pub static PAD_MASK_LEFT: u16 0x0006
pub static PAD_MASK_UPLEFT: u16 0x0007
pub static PAD_MASK_NONE: u16 0x0008

pub struct keydata {
    buttons: u16,
    dpad: u8,
    padding: u8,
    lx: u8,
    ly: u8,
    rx: u8,
    ry: u8,
}