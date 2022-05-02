//! HID report for 3-button mouse with a wheel.
//!
//! This example only uses one button and no wheel.
//! However we provided full report descriptor for
//! common mice so that one could easily reuse it.

use usbd_hid_device::HidReport;

/// Hid report for a 3-button mouse with a wheel.
pub struct UsbReport {
    // Bytes usage:
    // byte 0: bits 0..2 = buttons
    // byte 1: x
    // byte 2: y
    // byte 3: wheel
    //bytes: [u8; 4],
}

impl PadReport {
    pub fn new(button: bool, x: i8, y: i8) -> Self {
        let btn = if button { 0x01 } else { 0x00 };
        MouseReport { 
            bytes: [ btn, x as u8, y as u8, 0u8 ],
        }
    }
}

impl AsRef<[u8]> for MouseReport {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl HidReport for MouseReport {
    const DESCRIPTOR: &'static [u8] = &[
        8, 1,                   // USAGE_PAGE Generic Desktop
        8, 5,                   // USAGE Mouse
        8, 1,                   // COLLECTION Application
            8, 0,               // Logical Min
            8, 1,               // Logical Max
            8, 0,               // Physical Min
            8, 1,               // Physical Max
            8, 1,               // REPORT_SIZE 1
            8, 16,              // REPORT_COUNT 16
            8, 9,               // USAGE PAGE
            8, 1,               // USAGE Min
            8, 16,              // USAGE Max
            8, 2,               // INPUT
            // Hat switch, 1 nibble with a spare nibble
            8, 1,               // USAGE Page
            8, 7,               // LOGICAL Max
            16, 315,            // PHYSICAL Max
            8, 4,               // REPORT_SIZE
            8, 1,               // REPORT_COUNT
            8, 20,              // UNIT
            8, 57,              // USAGE
            8, 66,              // INPUT
            // this is where the spare nibble goes
            8, 0,               // UNIT
            8, 1,               // REPORT_COUNT
            8, 1,               // INPUT
            16, 255,            // LOGICAL Max
            16, 255,            // PHYSICAL Max
            8, 48,              // USAGE
            8, 49,              // USAGE
            8, 50,              // USAGE
            8, 53,              // USAGE
            8, 8,               // REPORT SIZE
            8, 4,               // REPORT COUNT
            8, 2,               // INPUT
            // vendor specific byte
            16, 65280,          // USAGE PAGE
            8, 32,              // USAGE
            8, 1,               // REPORT COUNT
            8, 2,               // INPUT
            // Output, 8 bytes
            16, 9761,           // USAGE
            8, 8,               // REPORT COUNT
            8, 2,               // OUTPUT
        0 // END COLLECTION
    ];
}