# MumenController
This project implements a fight stick firmware to be fast and compliant to evo rules. 

# Setup (i.MX RT/Teensy 4.0)
install [rustup ](https://rustup.rs/)

`rustup update stable`

`rustup target add thumbv7em-none-eabihf`

`cargo install toml-fmt cargo-generate`

`cargo objcopy --release -- -O ihex mumen.hex`

`teensy_loader_cli --mcu=IMXRT1062 -w mumen.hex`

# Pinout Configurations

The controller now supports multiple pinout configurations that can be selected at build time. This allows for building with different hardware layouts without modifying the code.

## Available Pinout Configurations

### Standard Pinout (Default)
This is the original pinout configuration with all pins enabled:
- All buttons (A, B, X, Y, L1, R1, L2, R2, L3, R3, Select, Start, Home)
- D-pad (Up, Down, Left, Right)
- Analog toggles (Left, Right)
- Lock button
- Analog sticks (Lx, Ly, Rx, Ry)

### Alternate Pinout
This configuration has the following changes:
- A and B buttons are inverted
- L2, R2, L3, and R3 are not configured
- Shift and Lock buttons are not configured
- Analog sticks (Lx, Ly, Rx, Ry) are not configured

## Building with Different Pinout Configurations

### Standard Pinout (Default)
```bash
cargo objcopy --release -- -O ihex mumen.hex
```

### Alternate Pinout
```bash
cargo objcopy --release --features "alternate_pinout" -- -O ihex mumen_alt.hex
```

When a pin is not configured in a particular pinout, it will report a neutral value (especially important for analog inputs which report 128 as neutral).

# Old Setup (atmega32u4)

## windows
install [VS 2022 build tools](https://visualstudio.microsoft.com/downloads/#other)

install [winavr](https://sourceforge.net/projects/winavr/files/latest/download)

## all platforms
install [arduino IDE](https://www.arduino.cc/en/software/)

install [rustup ](https://rustup.rs/)

`rustup set default nightly`

`rustup component add nightly rust-src`

`cargo +stable install ravedude toml-fmt cargo-generate`

# Ack

[lithe](https://github.com/konkers/lithe) was a great example. Thanks, konkers.

[ATMega32U4-Switch-Fightstick](https://github.com/fluffymadness/ATMega32U4-Switch-Fightstick) got me started and provided a great logical scaffolding and introduction to fight sticks on the Atmega 32U4. Thank you, fluffymadness. 

[avr-hal](https://github.com/rahix/avr-hal) is a great toolset for getting started in rust on most arduino boards. You rock, Rahix.

[usb-hid-device](https://github.com/agalakhov/usbd-hid-device) wildly accelerated the time it took to learn HID reporting and provides a clean API to work with HID descriptors. I appreciate your work, agalakhov.

[usb-device](https://github.com/rust-embedded-community/usb-device) provides a great framework which usb-hid-device was built on, but it doesn't work on this target out-of-the-box. Thank you, agausmann for your hard work bringing important features of rust to AVR wit projects like [this](https://github.com/agausmann/usb-device/tree/bd5a518dff4a688bed05c67c83fea733b69c9623). Maybe one day the conflicts can be resolved and your PR can be merged.

[Unflappable](https://github.com/couchand/unflappable) provides nice portable debounced pin objects. It would be nice to have the functionality it needs available in the stable rust. Keep up the great work, couchand.

I'd also like to thank the Embedded Rust community as a whole. I stand on the shoulders of giants. Check out [Awesome Embedded Rust](https://github.com/rust-embedded/awesome-embedded-rust) for more great projects to follow.
