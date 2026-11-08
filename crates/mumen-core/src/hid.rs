//! Nintendo Switch / HORI Pokken wired USB HID report descriptor and 8-byte report format.
//!
//! The descriptor and report format match the proven `UnlicProController` (`Descriptors.c`
//! and `Joystick.h`) layout so the Nintendo Switch recognizes the device as a wired licensed
//! controller without requiring a proprietary Bluetooth-over-USB handshake.

/// HORI Co., Ltd. USB Vendor ID used by licensed wired Switch controllers.
pub const HORI_VID: u16 = 0x0F0D;

/// HORI Pokken Tournament DX Pro Pad USB Product ID.
pub const HORI_POKKEN_PID: u16 = 0x0092;

/// Neutral analog stick value (`128` on `0..=255`).
pub const STICK_CENTER: u8 = 128;

/// Minimum analog stick value (`0`, Up on Y / Left on X).
pub const STICK_MIN: u8 = 0;

/// Maximum analog stick value (`255`, Down on Y / Right on X).
pub const STICK_MAX: u8 = 255;

/// Half-tilt low value (`64`) used in Smash mode when Shift is held.
pub const STICK_HALF_LOW: u8 = 64;

/// Half-tilt high value (`192`) used in Smash mode when Shift is held.
pub const STICK_HALF_HIGH: u8 = 192;

/// 86-byte USB HID Report Descriptor matching `UnlicProController/Descriptors.c`.
///
/// Layout of the 8-byte input report:
/// - Bytes 0..=1: 16 button bits (little-endian `u16`)
/// - Byte 2 (low nibble): 4-bit Hat switch (`0..=7` directions, `8` / `0x0F` released)
/// - Byte 2 (high nibble): 4-bit constant padding
/// - Byte 3: Left Stick X (`0..=255`, `128` center)
/// - Byte 4: Left Stick Y (`0..=255`, `128` center)
/// - Byte 5: Right Stick X (`0..=255`, `128` center)
/// - Byte 6: Right Stick Y (`0..=255`, `128` center)
/// - Byte 7: Vendor-specific byte (`0x00`)
///
/// Also declares the 8-byte Output report required by the Nintendo Switch during enumeration.
pub const REPORT_DESCRIPTOR: [u8; 86] = [
    0x05, 0x01, // USAGE_PAGE (Generic Desktop)
    0x09, 0x05, // USAGE (Joystick)
    0xA1, 0x01, // COLLECTION (Application)
    // Buttons (2 bytes = 16 bits)
    0x15, 0x00, //   LOGICAL_MINIMUM (0)
    0x25, 0x01, //   LOGICAL_MAXIMUM (1)
    0x35, 0x00, //   PHYSICAL_MINIMUM (0)
    0x45, 0x01, //   PHYSICAL_MAXIMUM (1)
    0x75, 0x01, //   REPORT_SIZE (1)
    0x95, 0x10, //   REPORT_COUNT (16)
    0x05, 0x09, //   USAGE_PAGE (Button)
    0x19, 0x01, //   USAGE_MINIMUM (Button 1)
    0x29, 0x10, //   USAGE_MAXIMUM (Button 16)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    // HAT Switch (1 nibble = 4 bits)
    0x05, 0x01, //   USAGE_PAGE (Generic Desktop)
    0x25, 0x07, //   LOGICAL_MAXIMUM (7)
    0x46, 0x3B, 0x01, // PHYSICAL_MAXIMUM (315)
    0x75, 0x04, //   REPORT_SIZE (4)
    0x95, 0x01, //   REPORT_COUNT (1)
    0x65, 0x14, //   UNIT (Eng Rot:Angular Pos)
    0x09, 0x39, //   USAGE (Hat switch)
    0x81, 0x42, //   INPUT (Data,Var,Abs,Null)
    // Spare nibble (4 bits constant)
    0x65, 0x00, //   UNIT (None)
    0x95, 0x01, //   REPORT_COUNT (1)
    0x81, 0x01, //   INPUT (Cnst,Ary,Abs)
    // Analog sticks (4 bytes: X, Y, Z, Rz)
    0x26, 0xFF, 0x00, // LOGICAL_MAXIMUM (255)
    0x46, 0xFF, 0x00, // PHYSICAL_MAXIMUM (255)
    0x09, 0x30, //   USAGE (X)
    0x09, 0x31, //   USAGE (Y)
    0x09, 0x32, //   USAGE (Z)
    0x09, 0x35, //   USAGE (Rz)
    0x75, 0x08, //   REPORT_SIZE (8)
    0x95, 0x04, //   REPORT_COUNT (4)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    // Vendor specific (1 byte)
    0x06, 0x00, 0xFF, // USAGE_PAGE (Vendor Defined 0xFF00)
    0x09, 0x20, //   USAGE (0x20)
    0x95, 0x01, //   REPORT_COUNT (1)
    0x81, 0x02, //   INPUT (Data,Var,Abs)
    // Output report (8 bytes)
    0x0A, 0x21, 0x26, // USAGE (0x2621)
    0x95, 0x08, //   REPORT_COUNT (8)
    0x91, 0x02, //   OUTPUT (Data,Var,Abs)
    0xC0, // END_COLLECTION
];

/// Bitmasks for the 16-bit button field in [`SwitchReport`].
///
/// Matches the bit assignments in `UnlicProController/UnlicProController.ino`
/// and `MumenController`'s original `spc.rs`.
pub struct ButtonMask;

impl ButtonMask {
    pub const NONE: u16 = 0x0000;
    pub const Y: u16 = 0x0001;
    pub const B: u16 = 0x0002;
    pub const A: u16 = 0x0004;
    pub const X: u16 = 0x0008;
    pub const L: u16 = 0x0010;
    pub const R: u16 = 0x0020;
    pub const ZL: u16 = 0x0040;
    pub const ZR: u16 = 0x0080;
    pub const MINUS: u16 = 0x0100;
    pub const PLUS: u16 = 0x0200;
    pub const L3: u16 = 0x0400;
    pub const R3: u16 = 0x0800;
    pub const HOME: u16 = 0x1000;
    pub const CAPTURE: u16 = 0x2000;

    /// Menu buttons suppressed when the tournament Lock switch is active.
    pub const LOCKABLE_MENU_MASK: u16 = Self::MINUS | Self::PLUS | Self::HOME | Self::CAPTURE;
}

/// 8-direction + neutral HAT switch encoding for the Switch HID report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Hat {
    Up = 0x00,
    UpRight = 0x01,
    Right = 0x02,
    DownRight = 0x03,
    Down = 0x04,
    DownLeft = 0x05,
    Left = 0x06,
    UpLeft = 0x07,
    Neutral = 0x08,
}

impl Hat {
    /// Convert cleaned `(up, down, left, right)` directional booleans into a [`Hat`] value.
    ///
    /// If opposing directions are somehow still both true, they cancel out.
    #[inline]
    pub const fn from_directions(up: bool, down: bool, left: bool, right: bool) -> Self {
        let u = up && !down;
        let d = down && !up;
        let l = left && !right;
        let r = right && !left;

        match (u, d, l, r) {
            (true, false, false, false) => Self::Up,
            (true, false, false, true) => Self::UpRight,
            (false, false, false, true) => Self::Right,
            (false, true, false, true) => Self::DownRight,
            (false, true, false, false) => Self::Down,
            (false, true, true, false) => Self::DownLeft,
            (false, false, true, false) => Self::Left,
            (true, false, true, false) => Self::UpLeft,
            _ => Self::Neutral,
        }
    }
}

/// 8-byte input report transmitted over USB HID to the Nintendo Switch or PC host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchReport {
    pub buttons: u16,
    pub hat: Hat,
    pub lx: u8,
    pub ly: u8,
    pub rx: u8,
    pub ry: u8,
    pub vendor: u8,
}

impl Default for SwitchReport {
    fn default() -> Self {
        Self::neutral()
    }
}

impl SwitchReport {
    /// Create a neutral report (no buttons pressed, hat centered, sticks at 128).
    #[inline]
    pub const fn neutral() -> Self {
        Self {
            buttons: ButtonMask::NONE,
            hat: Hat::Neutral,
            lx: STICK_CENTER,
            ly: STICK_CENTER,
            rx: STICK_CENTER,
            ry: STICK_CENTER,
            vendor: 0,
        }
    }

    /// Serialize the report into its exact 8-byte wire representation.
    #[inline]
    pub const fn to_bytes(&self) -> [u8; 8] {
        let [btn_lo, btn_hi] = self.buttons.to_le_bytes();
        [
            btn_lo,
            btn_hi,
            self.hat as u8,
            self.lx,
            self.ly,
            self.rx,
            self.ry,
            self.vendor,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_length_and_framing() {
        assert_eq!(REPORT_DESCRIPTOR.len(), 86);
        assert_eq!(
            REPORT_DESCRIPTOR[0..6],
            [0x05, 0x01, 0x09, 0x05, 0xA1, 0x01]
        );
        assert_eq!(*REPORT_DESCRIPTOR.last().unwrap(), 0xC0);
    }

    #[test]
    fn neutral_report_wire_bytes() {
        let report = SwitchReport::neutral();
        assert_eq!(
            report.to_bytes(),
            [0x00, 0x00, 0x08, 128, 128, 128, 128, 0x00]
        );
    }

    #[test]
    fn button_packing_is_little_endian() {
        let mut report = SwitchReport::neutral();
        report.buttons = ButtonMask::A | ButtonMask::Y | ButtonMask::HOME | ButtonMask::PLUS;
        let bytes = report.to_bytes();
        assert_eq!(bytes[0], (ButtonMask::A | ButtonMask::Y) as u8);
        assert_eq!(bytes[1], ((ButtonMask::HOME | ButtonMask::PLUS) >> 8) as u8);
    }

    #[test]
    fn hat_encoding_matches_switch_spec() {
        assert_eq!(Hat::from_directions(true, false, false, false), Hat::Up);
        assert_eq!(Hat::from_directions(true, false, false, true), Hat::UpRight);
        assert_eq!(Hat::from_directions(false, false, false, true), Hat::Right);
        assert_eq!(
            Hat::from_directions(false, true, false, true),
            Hat::DownRight
        );
        assert_eq!(Hat::from_directions(false, true, false, false), Hat::Down);
        assert_eq!(
            Hat::from_directions(false, true, true, false),
            Hat::DownLeft
        );
        assert_eq!(Hat::from_directions(false, false, true, false), Hat::Left);
        assert_eq!(Hat::from_directions(true, false, true, false), Hat::UpLeft);
        assert_eq!(
            Hat::from_directions(false, false, false, false),
            Hat::Neutral
        );
        assert_eq!(Hat::from_directions(true, true, false, false), Hat::Neutral);
    }
}
