# Goals
[x] port c++ PoC logic to rust
[x] implement input signal parsing
[x] support mode-switching
[ ] setup HID descriptor objects
[ ] transmit gamepad signals via usb
Take a peek at [usbd-human-interface-device|https://crates.io/crates/usbd-human-interface-device], it looks a lot better these days
Example [Joystick|https://github.com/dlkj/usbd-human-interface-device/blob/main/examples/src/bin/joystick.rs]
code for the [trait|https://github.com/dlkj/usbd-human-interface-device/blob/main/src/device/joystick.rs] to extend, `usbd_human_interface_device::device::joystick::JoystickConfig::default()`

# Versioning:
[ ] 0.0.2 HID connection established with host, pad reports communicated
[ ] 0.1.0 Test suite development started, essential positive general use unit tests exist
[ ] 0.2.0 Test suite established, identified bugs remediated
[ ] 1.0.0 CI/CD pipeline deployed