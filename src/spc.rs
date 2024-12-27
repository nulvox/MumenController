// Provide functionality to interact with USB devices.
// spc == Switch Pro Controller

// use usb_device::device::UsbDeviceBuilder;
// use usb_device::prelude::*;
// use usbd_hid_device::Hid::HidReport;
use usbd_hid::{descriptor, hid_class};
use usbd_hid_device::HidReport;
use usbd_hid_macros::gen_hid_descriptor;

/// Report types where serialized HID report descriptors are available.
pub trait SerializedDescriptor {
    fn desc() -> &'static [u8];
}

/// Report types which serialize into input reports, ready for transmission.
pub trait AsInputReport: Serialize {}

/// Prelude for modules which use the `gen_hid_descriptor` macro.
pub mod generator_prelude {
    pub use crate::descriptor::{AsInputReport, SerializedDescriptor};
    pub use serde::ser::{Serialize, SerializeTuple, Serializer};
    pub use usbd_hid_macros::gen_hid_descriptor;
}
use usbd_hid::descriptor::generator_prelude::{Serialize, SerializeTuple, Serializer};

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

// This feels wrong...
#[gen_hid_descriptor(
    (collection = APPLICATION, usage_page = GENERIC_DESKTOP, usage = 0x05) = {
            #[item_settings data,variable,packed_bits 14] buttons=input;
            #[item_settings data,variable] hat=input;
            #[item_settings data] padding=input;
            #[item_settings data,variable] lx=input;
            #[item_settings data,variable] ly=input;
            #[item_settings data,variable] rx=inpout;
            #[item_settings data,variable] ry=input;
    }
)]
#[allow(dead_code)]
pub struct KeyData {
    pub buttons: u16,
    pub hat: u8,
    pub padding: u8,
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
}

/// Hid report for a 3-button mouse with a wheel.
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
        let btnarray = btnstate.buttons.to_be_bytes();
        PadReport {
            bytes: [
                btnarray[0],
                btnarray[1],
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

// This pad report matches those produced by other nintendo Switch firght sticks
impl HidReport for PadReport {
    const DESCRIPTOR: &'static [u8] = &[
        0x08, 0x01, // USAGE_PAGE Generic Desktop
        0x08, 0x05, // USAGE Joystick
        0x08, 0x01, // COLLECTION Application
        0x08, 0x00, // Logical Min
        0x08, 0x01, // Logical Max
        0x08, 0x00, // Physical Min
        0x08, 0x01, // Physical Max
        0x08, 0x01, // REPORT_SIZE 1
        0x08, 0x10, // REPORT_COUNT 16
        0x08, 0x09, // USAGE PAGE
        0x08, 0x01, // USAGE Min
        0x08, 0x10, // USAGE Max
        0x08, 0x02, // INPUT
        // Hat switch, 1 nibble with a spare nibble
        0x08, 0x01, // USAGE Page
        0x08, 0x07, // LOGICAL Max
        0x10, 0x01, 0x3B, // PHYSICAL Max
        0x08, 0x04, // REPORT_SIZE
        0x08, 0x01, // REPORT_COUNT
        0x08, 0x14, // UNIT
        0x08, 0x39, // USAGE
        0x08, 0x42, // INPUT
        // this is where the spare nibble goes
        0x08, 0x00, // UNIT
        0x08, 0x01, // REPORT_COUNT
        0x08, 0x01, // INPUT
        0x10, 0xFF, 0xFF, // LOGICAL Max
        0x10, 255, // PHYSICAL Max
        0x08, 0x08, // USAGE
        0x08, 0x31, // USAGE
        0x08, 0x32, // USAGE
        0x08, 0x35, // USAGE
        0x08, 0x08, // REPORT SIZE
        0x08, 0x04, // REPORT COUNT
        0x08, 0x02, // INPUT
        // vendor specific byte
        0x10, 0xFF, 0x00, // USAGE PAGE  (16-bit value, this hack is ugly)
        0x08, 0x20, // USAGE
        0x08, 0x01, // REPORT COUNT
        0x08, 0x02, // INPUT
        // Output, 8 bytes
        0x10, 0x26, 0x21, // USAGE  (16-bit value, this hack is ugly)
        0x08, 0x08, // REPORT COUNT
        0x08, 0x02, // OUTPUT
        0x00, // END COLLECTION
    ];
}
