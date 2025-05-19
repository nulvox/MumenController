//! USB HID implementation for Nintendo Switch Pro controller
//!
//! This module handles all USB communication with the Nintendo Switch,
//! presenting the Teensy 4.0 as a Pro Controller.
//!
//! The implementation is based on the Nintendo Switch Pro controller
//! protocol with optimizations for low latency and reliability.

mod descriptor;
mod device;

// Re-export public components
pub use descriptor::SwitchProReport;
pub use device::SwitchProDevice;