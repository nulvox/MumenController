//! HID descriptor for Nintendo Switch Pro controller
//!
//! This module implements the HID descriptor and report format
//! for the Nintendo Switch Pro controller.

use usbd_hid::descriptor::SerializedDescriptor;
use usbd_hid::descriptor::AsInputReport;

/// Nintendo Switch Pro controller HID report descriptor
///
/// This descriptor enables HID functionality for the Nintendo Switch Pro controller
/// based on the C implementation in descriptors.c
pub struct SwitchProReportDescriptor {}

// Implement the SerializedDescriptor trait required by usbd-hid
impl SerializedDescriptor for SwitchProReportDescriptor {
    fn desc() -> &'static [u8] {
        // This is a complete and accurate version of the HID report descriptor
        // for the Nintendo Switch Pro Controller
        // Based on the descriptor in descriptors.c
        static DESCRIPTOR: [u8; 76] = [
            0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
            0x09, 0x05,        // USAGE (Joystick)
            0xA1, 0x01,        // COLLECTION (Application)
            // Buttons (2 bytes)
            0x15, 0x00,        // LOGICAL_MINIMUM (0)
            0x25, 0x01,        // LOGICAL_MAXIMUM (1)
            0x75, 0x01,        // REPORT_SIZE (1)
            0x95, 0x10,        // REPORT_COUNT (16)
            0x05, 0x09,        // USAGE_PAGE (Button)
            0x19, 0x01,        // USAGE_MINIMUM (Button 1)
            0x29, 0x10,        // USAGE_MAXIMUM (Button 16)
            0x81, 0x02,        // INPUT (Data,Var,Abs)
            // HAT switch (4 bits)
            0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
            0x25, 0x07,        // LOGICAL_MAXIMUM (7)
            0x46, 0x3B, 0x01,  // PHYSICAL_MAXIMUM (315)
            0x75, 0x04,        // REPORT_SIZE (4)
            0x95, 0x01,        // REPORT_COUNT (1)
            0x65, 0x14,        // UNIT (Eng Rot:Angular Pos)
            0x09, 0x39,        // USAGE (Hat switch)
            0x81, 0x42,        // INPUT (Data,Var,Abs,Null)
            // Reserved (4 bits)
            0x75, 0x04,        // REPORT_SIZE (4)
            0x95, 0x01,        // REPORT_COUNT (1)
            0x81, 0x03,        // INPUT (Cnst,Var,Abs)
            // Analog sticks (4 bytes)
            0x15, 0x00,        // LOGICAL_MINIMUM (0)
            0x25, 0xFF,        // LOGICAL_MAXIMUM (255)
            0x75, 0x08,        // REPORT_SIZE (8)
            0x95, 0x04,        // REPORT_COUNT (4)
            0x05, 0x01,        // USAGE_PAGE (Generic Desktop)
            0x09, 0x30,        // USAGE (X)
            0x09, 0x31,        // USAGE (Y)
            0x09, 0x32,        // USAGE (Z)
            0x09, 0x35,        // USAGE (Rz)
            0x81, 0x02,        // INPUT (Data,Var,Abs)
            // Vendor specific (1 byte)
            0x15, 0x00,        // LOGICAL_MINIMUM (0)
            0x25, 0xFF,        // LOGICAL_MAXIMUM (255)
            0x75, 0x08,        // REPORT_SIZE (8)
            0x95, 0x01,        // REPORT_COUNT (1)
            0x81, 0x03,        // INPUT (Cnst,Var,Abs)
            0xC0               // END_COLLECTION
        ];
        
        &DESCRIPTOR
    }
}

/// Nintendo Switch Pro controller HID report
/// 
/// This struct represents the input report sent to the host
#[derive(Debug, Clone, Copy)]
pub struct SwitchProReport {
    /// 16 buttons (A, B, X, Y, etc.)
    pub buttons: [bool; 16],
    /// HAT/D-pad direction (0-7, 8 = released)
    pub hat: u8,
    /// Left stick X coordinate
    pub left_stick_x: u8,
    /// Left stick Y coordinate
    pub left_stick_y: u8,
    /// Right stick X coordinate
    pub right_stick_x: u8,
    /// Right stick Y coordinate
    pub right_stick_y: u8,
    /// Vendor specific data
    pub vendor_spec: u8,
}

impl SwitchProReport {
    /// Create a new Switch Pro controller report with default values
    pub fn new() -> Self {
        Self {
            buttons: [false; 16],
            hat: 8, // 8 represents no HAT input
            left_stick_x: 128, // Center position
            left_stick_y: 128, // Center position
            right_stick_x: 128, // Center position
            right_stick_y: 128, // Center position
            vendor_spec: 0,
        }
    }
    
    /// Convert the report to a byte array for USB HID
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut result = [0; 8];
        
        // Pack buttons into 2 bytes
        let mut buttons_bytes = [0u8; 2];
        for i in 0..16 {
            if self.buttons[i] {
                if i < 8 {
                    buttons_bytes[0] |= 1 << i;
                } else {
                    buttons_bytes[1] |= 1 << (i - 8);
                }
            }
        }
        
        result[0] = buttons_bytes[0];
        result[1] = buttons_bytes[1];
        
        // Pack HAT/D-pad - use first 4 bits of byte 2
        // If hat is 8 (released), use 0x0F which represents no direction
        let hat_value = if self.hat <= 7 { self.hat } else { 0x0F };
        result[2] = hat_value & 0x0F;  // Reserved bits are 0
        
        // Analog sticks
        result[3] = self.left_stick_x;
        result[4] = self.left_stick_y;
        result[5] = self.right_stick_x;
        result[6] = self.right_stick_y;
        
        // Vendor specific
        result[7] = self.vendor_spec;
        
        result
    }
    
    /// Set a button state by index (0-15)
    pub fn set_button(&mut self, index: usize, pressed: bool) {
        if index < self.buttons.len() {
            self.buttons[index] = pressed;
        }
    }
    
    /// Set HAT/D-pad direction
    /// 0 = N, 1 = NE, 2 = E, 3 = SE, 4 = S, 5 = SW, 6 = W, 7 = NW, 8 = Released
    pub fn set_hat(&mut self, direction: u8) {
        self.hat = if direction <= 7 { direction } else { 8 };
    }
    
    /// Set analog stick values
    pub fn set_left_stick(&mut self, x: u8, y: u8) {
        self.left_stick_x = x;
        self.left_stick_y = y;
    }
    
    /// Set right analog stick values
    pub fn set_right_stick(&mut self, x: u8, y: u8) {
        self.right_stick_x = x;
        self.right_stick_y = y;
    }
    
    /// Set vendor specific data
    pub fn set_vendor_data(&mut self, data: u8) {
        self.vendor_spec = data;
    }
    
    // No to_report method needed anymore
}