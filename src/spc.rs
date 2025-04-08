// Provide functionality to interact with USB devices.
// spc == Switch Pro Controller

// use usb_device::device::UsbDeviceBuilder;
// use usb_device::prelude::*;
use serde::Serialize;
// Import used for the trait impl but not directly
use usbd_hid_device::HidReport;

/// Report types where serialized HID report descriptors are available.
pub trait SerializedDescriptor {
    fn desc() -> &'static [u8];
}

/// Report types which serialize into input reports, ready for transmission.
// Remove our custom trait and use the one from usbd_hid
// Implementation for both types
impl usbd_hid::descriptor::AsInputReport for PadReport {}
impl usbd_hid::descriptor::AsInputReport for KeyData {}

pub const KEY_MASK_A: u16 = 0x0004;
pub const KEY_MASK_B: u16 = 0x0002;
pub const KEY_MASK_X: u16 = 0x0008;
pub const KEY_MASK_Y: u16 = 0x0001;
pub const KEY_MASK_L1: u16 = 0x0010;
pub const KEY_MASK_R1: u16 = 0x0020;
pub const KEY_MASK_L2: u16 = 0x0040;
pub const KEY_MASK_R2: u16 = 0x0080;
pub const KEY_MASK_L3: u16 = 0x0400;
pub const KEY_MASK_R3: u16 = 0x0800;
pub const KEY_MASK_SELECT: u16 = 0x0100;
pub const KEY_MASK_START: u16 = 0x0200;
pub const KEY_MASK_HOME: u16 = 0x1000;
// pub const KEY_MASK_CAP: u16 = 0x2000;

const JOYSTICK: u16 = 0x0805;

// Nintendo uses a hat switch to mark
// dpad inputs which force SOCD cleaning in firmware
// this makes processing take forever, so we are mapping
// the hat switch values to our array offsets and
// recording the data as masks to speed up processing
pub const HAT_MASK_UP: u8 = 0x01;
pub const HAT_MASK_DOWN: u8 = 0x02;
pub const HAT_MASK_LEFT: u8 = 0x04;
pub const HAT_MASK_RIGHT: u8 = 0x08;
const HAT_VALUES: [u8; 9] = [
    1,  // up
    9,  // up-right
    8,  // right
    10, // down-right
    2,  // down
    6,  // down-left
    4,  // left
    5,  // up-left
    0,  // neutral
];
// Must account for every SOCD combination in one of these arrays.
const SOCD_CODES: [u8; 7] = [
    3,  // Up + Down
    7,  // Sans-right
    11, // Sans-left
    12, // Left + Right
    13, // Sans-down
    14, // Sans-up
    15, // All
];
// Define KeyData struct directly without macro
#[derive(Clone, Copy, Serialize)]
#[allow(dead_code)]
#[repr(C, packed)]
pub struct KeyData {
    pub buttons: u16,
    pub hat: u8,
    pub padding: u8,
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
}

// KeyData implements AsRef<[u8]> required for usbd-hid
impl AsRef<[u8]> for KeyData {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// Hid report for a 3-button mouse with a wheel.
#[derive(Clone, Copy, Serialize)]
pub struct PadReport {
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
    pub fn new(btnstate: &KeyData) -> Self {
        // Use little-endian byte order for USB HID compatibility
        let btnarray = btnstate.buttons.to_le_bytes();
        
        // Create a properly formatted report
        PadReport {
            bytes: [
                btnarray[0], // Low byte of buttons
                btnarray[1], // High byte of buttons
                btnstate.hat,
                0x00, // padding for hat switch
                btnstate.lx,
                btnstate.ly,
                btnstate.rx,
                btnstate.ry,
            ],
        }
    }

    pub fn clear_keys(&mut self) {
        self.bytes[0] = 0;
        self.bytes[1] = 0;
        self.bytes[2] = 0;
        // left stick set to neutral
        self.bytes[4] = 128;
        self.bytes[5] = 128;
        // right stick set to neutral
        self.bytes[6] = 128;
        self.bytes[7] = 128;
    }

    // For ease of processing, we use bitfields when measuring the dpad
    // Nintendo uses a special hat encoding which implicitly scrubs SOCD
    // to support that with minimal effort, this helper function sets the
    // hat switch according to our directional bitfields
    pub fn set_hat(&mut self, dpad: u8) {
        // This is where we process SOCD using the following table.
        // map to the list in HAT_VALUES around line 53
        // 3 == up+down
        // 7 == up+down+left
        // 15 == all directions
        // 14 == sans-up
        // 13 == sans-down
        // 12 == left+right
        // 11 == right+up+down
        let clean_dpad = match dpad {
            3 => 1,       // up from up+down
            7 | 13 => 5,  // up-left from sans-right and sans-down
            11 | 15 => 9, // up-right from all-keys and sans-left
            12 => 4,      // left from left+right (tekken left-side preference)
            14 => 6,      // down-left from sans-up
            _ => dpad,    // if we got anything else, it's already clean...?
        };
        self.bytes[2] = HAT_VALUES.iter().position(|&r| r == clean_dpad).unwrap() as u8;
    }

    pub fn send(&self) {
        return;
    }
}

impl AsRef<[u8]> for PadReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

// Implement HidReport for KeyData
impl HidReport for KeyData {
    // Fixed descriptor that properly handles hat switch and matches our data structure
    const DESCRIPTOR: &'static [u8] = &[
        0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
        0x09, 0x05,        // USAGE (Gamepad)
        0xA1, 0x01,        // COLLECTION (Application)
        
        // Buttons (16 bits)
        0x15, 0x00,        // LOGICAL_MINIMUM (0)
        0x25, 0x01,        // LOGICAL_MAXIMUM (1)
        0x75, 0x01,        // REPORT_SIZE (1)
        0x95, 0x10,        // REPORT_COUNT (16)
        0x05, 0x09,        // USAGE_PAGE (Button)
        0x19, 0x01,        // USAGE_MINIMUM (Button 1)
        0x29, 0x10,        // USAGE_MAXIMUM (Button 16)
        0x81, 0x02,        // INPUT (Data,Var,Abs)
        
        // Hat switch (8 bits)
        0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
        0x09, 0x39,        // USAGE (Hat switch)
        0x15, 0x00,        // LOGICAL_MINIMUM (0)
        0x25, 0x07,        // LOGICAL_MAXIMUM (7)
        0x35, 0x00,        // PHYSICAL_MINIMUM (0)
        0x46, 0x3B, 0x01,  // PHYSICAL_MAXIMUM (315)
        0x65, 0x14,        // UNIT (Eng Rot:Angular Pos)
        0x75, 0x08,        // REPORT_SIZE (8)
        0x95, 0x01,        // REPORT_COUNT (1)
        0x81, 0x02,        // INPUT (Data,Var,Abs)
        
        // Padding (8 bits)
        0x75, 0x08,        // REPORT_SIZE (8)
        0x95, 0x01,        // REPORT_COUNT (1)
        0x81, 0x03,        // INPUT (Cnst,Var,Abs)
        
        // Analog sticks (4 axes, 8 bits each)
        0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
        0x09, 0x30,        // USAGE (X) - Left stick X
        0x09, 0x31,        // USAGE (Y) - Left stick Y
        0x09, 0x32,        // USAGE (Z) - Right stick X
        0x09, 0x35,        // USAGE (Rz) - Right stick Y
        0x15, 0x00,        // LOGICAL_MINIMUM (0)
        0x26, 0xFF, 0x00,  // LOGICAL_MAXIMUM (255)
        0x75, 0x08,        // REPORT_SIZE (8)
        0x95, 0x04,        // REPORT_COUNT (4)
        0x81, 0x02,        // INPUT (Data,Var,Abs)
        
        0xC0               // END_COLLECTION
    ];
}

// Make PadReport use the same descriptor as KeyData
impl HidReport for PadReport {
    const DESCRIPTOR: &'static [u8] = KeyData::DESCRIPTOR;
}

