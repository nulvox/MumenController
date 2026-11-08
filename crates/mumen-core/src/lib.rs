//! Hardware-agnostic core logic for the MumenController fight stick.
//!
//! This crate is `no_std`, performs zero heap allocations, uses no `unsafe` code,
//! and has no MCU/HAL dependencies so all input processing and HID report encoding
//! can be unit-tested on the host and shared across Teensy 4.0 and ATmega32U4 targets.

#![no_std]
#![forbid(unsafe_code)]

pub mod debounce;
pub mod hid;
pub mod input;
pub mod socd;

pub use debounce::Debouncer;
pub use hid::{
    ButtonMask, Hat, SwitchReport, HORI_POKKEN_PID, HORI_VID, REPORT_DESCRIPTOR, STICK_CENTER,
    STICK_HALF_HIGH, STICK_HALF_LOW, STICK_MAX, STICK_MIN,
};
pub use input::{ControllerEngine, DirectionalMode, LogicalInputs};
pub use socd::{DpadRaw, SocdMethod, SocdState};
