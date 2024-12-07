// Provide functionality to interact with USB devices.

use usbd_hid_device::HidReport;

#[derive(Debug, Copy, Clone)]
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
