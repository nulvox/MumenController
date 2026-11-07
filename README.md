# MumenController

`MumenController` is an experimental `no_std` Rust firmware project for building a wired Nintendo Switch-style fight stick on a Teensy 4.0. It combines compile-time TOML pin/SOCD configuration, an RTIC application, input debouncing and filtering, and a custom USB HID report path.

> **Project status:** the USB descriptor, report packing, configuration generation, and controller loop are implemented, but the GPIO and ADC read functions are still placeholders in the current source. Treat this repository as active firmware development rather than a ready-to-flash controller release.

## Architecture

```text
Buttons / D-pad ──> GPIO scan ──> debounce ──> SOCD / lock logic ─┐
Analog sticks ────> ADC scan ───> filter + deadzone ──────────────┤
                                                                  v
Teensy 4.0 / RTIC task ──> 8-byte Switch-style HID report ──> USB host
        ^                                                        │
        └──────── USB interrupt handling and connection state ───┘
```

Pin and SOCD profiles are read from `config/` by `build.rs` and converted into Rust constants at compile time.

## Timing model: 1 ms intervals, not a latency guarantee

Two separate parts of the source target a 1 ms cadence:

- `src/usb/device.rs` passes `1` as the HID interrupt endpoint polling interval (`bInterval`), allowing the USB host to poll at a nominal **1000 Hz / 1 ms** interval when the host and bus schedule permit.
- `src/main.rs` delays the RTIC controller loop by `1.millis()` after each input-processing and report-send attempt, targeting approximately one scan/report cycle per millisecond plus execution and scheduling time.

These settings describe the requested USB report interval and firmware loop design. They do **not** prove sub-millisecond end-to-end input latency: actual latency also includes switch debounce, loop execution, USB scheduling, host processing, display/game timing, and the unfinished hardware input implementation.

## Teensy 4.0 pinout and wiring

The table below is generated from the checked-in defaults in [`config/pinout/default.toml`](config/pinout/default.toml).

| Input | Teensy 4.0 pin | Suggested wiring |
|---|---:|---|
| A / B / X / Y | 2 / 3 / 4 / 5 | Momentary switch between GPIO and GND; firmware is intended to use pull-ups |
| L / R | 6 / 7 | Momentary switch between GPIO and GND |
| ZL / ZR | 8 / 9 | Momentary switch between GPIO and GND |
| Plus / Minus | 10 / 11 | Momentary switch between GPIO and GND |
| Home | 12 | Momentary switch between GPIO and GND |
| Capture | 14 | Momentary switch between GPIO and GND |
| L3 / R3 | 15 / 16 | Momentary switch between GPIO and GND |
| D-pad Up / Down | 17 / 18 | Direction switch between GPIO and GND |
| D-pad Left / Right | 19 / 20 | Direction switch between GPIO and GND |
| Left stick X / Y | 23 (A10) / 22 (A9) | Potentiometer wiper to analog pin; outer terminals to 3.3 V and GND |
| Right stick X / Y | 21 (A8) / 20 (A7) | Potentiometer wiper to analog pin; outer terminals to 3.3 V and GND |
| Menu lock | 33 | Switch input used to mask Home, Plus, Minus, and Capture |
| Status LED | 13 | Teensy 4.0 onboard LED |

**Pin conflict in the default profile:** pin 20 is assigned to both D-pad Right and Right Stick Y. Choose one function or edit `config/pinout/default.toml` before wiring hardware. Keep all input voltages within the Teensy 4.0's 3.3 V logic limits and share a common ground.

## Features under development

- Nintendo Switch-style 8-byte USB HID input reports
- Compile-time button, analog input, lock-pin, and SOCD configuration
- Neutral, first-input, last-input, up-priority, and second-input SOCD modes
- Digital debounce and analog filtering/deadzone modules
- RTIC task structure with USB interrupt handling
- LED-based startup and panic diagnostics

## Build

### Prerequisites

Install Rust, the Cortex-M target, and the binary tools used to create a Teensy Loader image:

```sh
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils
```

The repository does not currently contain a `.cargo/config.toml`, so specify the target explicitly:

```sh
git clone https://github.com/nulvox/MumenController.git
cd MumenController

cargo fmt --all -- --check
cargo check --target thumbv7em-none-eabihf
cargo build --release --target thumbv7em-none-eabihf
cargo objcopy --release --target thumbv7em-none-eabihf -- -O ihex MumenController.hex
```

Load `MumenController.hex` with [Teensy Loader](https://www.pjrc.com/teensy/loader.html) after the hardware-facing input implementation is complete and verified.

## Configuration

Edit the default profiles before building:

- [`config/pinout/default.toml`](config/pinout/default.toml) — digital, analog, and lock-pin assignments
- [`config/socd/default.toml`](config/socd/default.toml) — opposite-direction resolution rules

The current `build.rs` reads the `default` profiles directly. Additional profile selection via environment variables is not yet wired into the build script.

## Related work

- [`nulvox/UnlicProController`](https://github.com/nulvox/UnlicProController) is the earlier AVR C fight-stick implementation for the SparkFun Pro Micro / ATmega32U4. `MumenController` carries the controller concept into an embedded Rust and Teensy 4.0 design.
- [`nulvox/usbd-human-interface-device`](https://github.com/nulvox/usbd-human-interface-device) is a companion fork used for experimentation with additional device-side USB HID definitions. The current `MumenController` manifest uses the crates.io `usbd-hid` package directly, so the fork is related context rather than a checked-in dependency.

## Documentation

- [Quick start](docs/QUICK_START.md)
- [Technical notes](docs/TECHNICAL.md)
- [Testing](docs/TESTING.md)
- [Debugging](docs/DEBUG.md)

## License

No `LICENSE` file is currently included. A license should be selected explicitly before one is added.
