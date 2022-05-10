// use usbd_hid_device::HidReport;
// use usbd_human_interface_device::prelude::*;
use usbd_human_interface_device::switch_gamepad::*;

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

/// Hid report for a switch gamepad
// pub struct PadReport {
//     // Bytes usage:
//     // byte 0..1: bits 0..13 = buttons, 14 and 15 are unused at this time
//     // byte 2: dpad hat switch
//     // byte 3: padding for hat switch
//     // byte 4: L stick X
//     // byte 5: L stick Y
//     // byte 6: R stick X
//     // byte 7: R stick Y
//     bytes: [u8; 8],
// }

impl PadReport {

    pub fn send(&self) {
        return
    }
}
