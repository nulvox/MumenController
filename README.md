# MumenController

`MumenController` is a multi-target `no_std` Rust firmware workspace for wired Nintendo Switch fight sticks and arcade controllers, supporting both the **Teensy 4.0** (ARM Cortex-M7) and the **SparkFun Pro Micro / Arduino Leonardo** (ATmega32U4).

It implements the licensed HORI Pokken Tournament DX Pro Pad USB HID report format (`0x0F0D:0x0092`, 86-byte report descriptor, 8-byte input and output reports) first proven in [`UnlicProController`](https://github.com/nulvox/UnlicProController), allowing the Nintendo Switch and PC hosts to recognize the controller natively over USB without proprietary handshakes.

---

## Workspace Architecture

```text
MumenController/
├── crates/
│   ├── mumen-core/          # Hardware-agnostic no_std core (#![forbid(unsafe_code)], host-tested)
│   │   └── src/
│   │       ├── hid.rs       # 86-byte HORI/Switch HID report descriptor & 8-byte SwitchReport
│   │       ├── socd.rs      # Neutral, PrimaryPriority (Up/Left), and LastWin SOCD resolvers
│   │       ├── debounce.rs  # Compact shift-register switch debouncer (2 bytes/switch)
│   │       └── input.rs     # LogicalInputs, Dpad/Analog/Smash modes, Shift & Lock handling
│   ├── firmware-teensy40/   # Teensy 4.0 (NXP i.MX RT1062, Cortex-M7) RTIC v2 firmware
│   │   └── src/
│   │       ├── main.rs      # USB_OTG1 High-Speed HID ISR + 1 ms Systick scan task
│   │       └── pins.rs      # Real iomuxc + gpio::Input / ADC1 wiring across 5 layout features
│   └── firmware-promicro/   # SparkFun Pro Micro / Leonardo (ATmega32U4) AVR firmware
│       └── src/
│           ├── main.rs      # 48 MHz PLL init + USB_GEN/USB_COM interrupt-driven HID
│           └── pins.rs      # 16 pull-up switch inputs + Red/Blue mode indicator LEDs
└── .github/workflows/ci.yml # zizmor security audit, mumen-core tests, Teensy & AVR release builds
```

### Core Features (`mumen-core`)
- **Zero-Allocation, Hardware-Agnostic Engine**: Pure `no_std` Rust with `#![forbid(unsafe_code)]` and zero HAL dependencies, unit-tested on the host (`cargo test -p mumen-core`).
- **SOCD Cleaning**: Configurable Simultaneous Opposing Cardinal Directions resolution (`Neutral`, `PrimaryPriority` for Up/Left priority, and edge-tracked `LastWin`). Default HitBox profile resolves `Left + Right -> Neutral` and `Up + Down -> Up`.
- **Directional Modes**:
  - **Hardware Toggle Switches** (Teensy 4.0 `layout-standard`): Dedicated toggle inputs route directional switches to the D-pad (HAT), Left Stick, or Right Stick.
  - **Software Mode Cycling** (`SHIFT + HOME` rising-edge chord on Pro Micro and `layout-d16s`): Cycles between `Dpad`, `Analog` (Left Stick, or Right Stick while `Shift` is held), and `Smash` (full stick throw `0`/`255`, or half-tilt `64`/`192` walk/tilt modifier while `Shift` is held).
- **Tournament Lock Switch**: Suppresses menu buttons (`Plus`/`Start`, `Minus`/`Select`, `Home`, `Capture`) when engaged to prevent accidental tournament disqualifications.
- **Prompt-Detection (Eager) Debouncing**: Zero-tick edge delay (`0 µs` added latency on switch closure) with a 4 ms contact-bounce lockout window, modeled on the positive-edge response analyzed by Loïc *WydD* Petit on [inputlag.science](https://inputlag.science/controller).

---

## Low-Latency Engineering & `inputlag.science` Optimization

The input pipeline in `MumenController` is designed around the controller latency methodology and empirical findings published by **Loïc (*WydD*) Petit** at [**inputlag.science**](https://inputlag.science/controller):

1. **Zero-Delay Eager ("Prompt-Detection") Debouncing (`crates/mumen-core/src/debounce.rs`)**:
   - As demonstrated on [inputlag.science/controller/results](https://inputlag.science/controller/results) (comparing the *Brook UFB Positive Edge* at **1.0 ms avg / 95% on-time** vs. *Negative Edge* at **3.9 ms avg / 78% on-time**), a traditional 4-sample shift-register debouncer adds a deterministic 3–4 ms delay before reporting a button press.
   - `mumen_core::Debouncer` instead transitions **immediately on the first sample (`0` ticks of delay)** when the lockout timer is `0`, and then locks out subsequent mechanical contact bounce for 4 ms (`20 × 200 µs` ticks on Teensy 4.0; `4 × 1 ms` SOF ticks on ATmega32U4).
2. **Sub-Millisecond (`200 µs` / 5 kHz) Scanning & Immediate USB Endpoint Priming (`crates/firmware-teensy40`)**:
   - [inputlag.science/controller](https://inputlag.science/controller) notes: *"Every additional sampling is bad for lag. Try to keep the polling of the actual button either using a synchronous method on the USB polling or using a fast sampling (500µs is enough)."*
   - On Teensy 4.0, the RTIC `Systick` monotonic runs at **10 kHz** and `scan_inputs` samples all GPIO pads every **`200 µs` (5 kHz)** — 2.5× faster than the 500 µs recommendation — bounding internal sampling jitter to $[0, 0.2]\,\text{ms}$ (`0.1 ms` average).
   - Whenever `scan_inputs` detects a state change (`next_report != prev_report`), it immediately primes the USB IN endpoint if idle, and the `USB_OTG1` ISR immediately re-primes the IN endpoint with the latest 200 µs sample as soon as the host consumes a packet.
   - For the analog stick layout (`layout-a4d17`), `ADC1` is configured for native 8-bit resolution (`ResolutionBits::Res8`), 1-sample hardware averaging (`AveragingCount::Avg1`), and `ConversionSpeed::Fast` to minimize blocking conversion overhead.
3. **1000 Hz (`bInterval = 1`) Native USB Polling & Synchronous Sampling (`crates/firmware-promicro`)**:
   - Both `firmware-teensy40` and `firmware-promicro` declare `bInterval = 1` in the USB HID endpoint descriptor, requesting the fastest legal polling rate under both Full-Speed (`1 ms` / 1000 Hz) and High-Speed (`125 µs` / 8000 Hz microframe) USB 2.0 schedulers.
   - On the ATmega32U4 Pro Micro, switches are sampled synchronously inside the 1 kHz USB SOF / endpoint interrupt right before loading the 8-byte report into the hardware FIFO.

### Latency Budget Summary

| Stage | Teensy 4.0 (`firmware-teensy40`) | Pro Micro (`firmware-promicro`) | Reference |
|---|---|---|---|
| **Switch Debounce Delay** | **0.0 ms** (4 ms bounce lockout) | **0.0 ms** (4 ms bounce lockout) | [inputlag.science](https://inputlag.science/controller/results/brook-ufb-ps4-positive-edge) |
| **Internal Scan Interval** | **0.20 ms** (5 kHz RTIC task) | **Synchronous** on 1 kHz USB SOF | [inputlag.science](https://inputlag.science/controller) ($\le 500\,\mu\text{s}$ rule) |
| **USB Endpoint `bInterval`** | **`1`** (125 µs HS / 1 ms FS) | **`1`** (1 ms FS / 1000 Hz) | [inputlag.science Methodology](https://inputlag.science/controller/methodology) |
| **Theoretical Avg Lag (1 kHz host)** | **~0.6 ms** (~96% on-time @ 60 fps) | **~0.5–1.0 ms** (~94–96% on-time @ 60 fps) | 16.67 ms frame window |

---

## Hardware Pinout & Layout Profiles

All momentary button and directional switches are wired between the MCU pin and **GND** (using internal pull-up resistors).

### Teensy 4.0 (`crates/firmware-teensy40`)

Select a layout at compile time with `--no-default-features --features <layout>`:

| Input | `layout-standard` *(default)* | `layout-alternate` | `layout-d16` | `layout-d16s` | `layout-a4d17` |
|---|---:|---:|---:|---:|---:|
| **A** | P14 | P11 | P0 | P0 | P0 |
| **B** | P11 | P14 | P1 | P1 | P1 |
| **X** | P9 | P9 | P2 | P2 | P2 |
| **Y** | P16 | P16 | P3 | P3 | P3 |
| **L (L1)** | P15 | P15 | P4 | P4 | P4 |
| **R (R1)** | P10 | P10 | P5 | P5 | P5 |
| **ZL (L2)** | P12 | — | P6 | P6 | P6 |
| **ZR (R2)** | P13 | — | P7 | P7 | P7 |
| **L3** | P3 | — | — | — | P11 |
| **R3** | P2 | — | — | — | P12 |
| **Minus (Select)** | P18 | P18 | P8 | P8 | P8 |
| **Plus (Start)** | P17 | P17 | P9 | P9 | P9 |
| **Home** | P8 | P8 | P10 | P10 | P10 |
| **Capture** | — | — | P11 | — | — |
| **Shift** | — | — | — | P11 | — |
| **D-pad Up** | P1 | P1 | P12 | P12 | P13 |
| **D-pad Down** | P6 | P6 | P13 | P13 | P14 |
| **D-pad Left** | P7 | P7 | P14 | P14 | P15 |
| **D-pad Right** | P19 | P19 | P15 | P15 | P16 |
| **Analog Left Toggle** | P4 | — | — | — | — |
| **Analog Right Toggle** | P5 | — | — | — | — |
| **Tournament Lock** | P0 *(100k pull-down)* | — | — | — | — |
| **Left Stick X / Y (ADC1)** | — | — | — | — | P17 / P18 |
| **Right Stick X / Y (ADC1)** | — | — | — | — | P19 / P20 |

### SparkFun Pro Micro / Arduino Leonardo (`crates/firmware-promicro`)

Select a layout with `--no-default-features --features <layout>`:
- `layout-promicro-standard` *(default)*: Matches `UnlicProController`.
- `layout-promicro-classic`: Permutes `L/R/ZL/ZR` and `Minus/Plus/Home` masks to match the legacy `origin/hid` wiring.

| Input / Indicator | Pro Micro Pin | ATmega32U4 Port | Notes |
|---|---|---|---|
| **A** | `D3` | `PD0` | Internal pull-up (active low to GND) |
| **B** | `D1` (`TX0`) | `PD3` | Internal pull-up |
| **X** | `A0` | `PF7` | Internal pull-up |
| **Y** | `D15` (`SCK`) | `PB1` | Internal pull-up |
| **L (LB)** | `A1` | `PF6` | Internal pull-up |
| **R (RB)** | `D5` | `PC6` | Internal pull-up |
| **ZL (LT)** | `A2` | `PF5` | Internal pull-up |
| **ZR (RT)** | `D0` (`RX1`) | `PD2` | Internal pull-up |
| **Minus (Select)** | `D14` (`MISO`) | `PB3` | Internal pull-up |
| **Plus (Start)** | `D10` | `PB6` | Internal pull-up |
| **Home** | `D16` (`MOSI`) | `PB2` | Internal pull-up |
| **Shift** | `D2` | `PD1` | Internal pull-up (`SHIFT + HOME` cycles modes) |
| **Up / Down / Left / Right** | `D7 / D8 / D6 / D9` | `PE6 / PB4 / PD7 / PB5` | Internal pull-up |
| **Red Mode LED** | `A3` | `PF4` | Output (`Analog` = Red, `Smash` = Red+Blue) |
| **Blue Mode LED** | `D4` | `PD4` | Output (`Dpad` = Blue, `Smash` = Red+Blue) |

---

## Building, Testing, and Flashing

### 1. Run Host Unit Tests (`mumen-core`)

```sh
cargo test -p mumen-core
```

### 2. Build & Flash Teensy 4.0 (`firmware-teensy40`)

```sh
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
cargo install cargo-binutils

cd crates/firmware-teensy40

# Build default layout (layout-standard), or pass --no-default-features --features layout-d16s
cargo build --release

# Convert ELF to Intel HEX for Teensy Loader
cargo objcopy --release -- -O ihex mumen-teensy40.hex
```

Flash `mumen-teensy40.hex` using [Teensy Loader](https://www.pjrc.com/teensy/loader.html) or `teensy_loader_cli --mcu=TEENSY40 -w mumen-teensy40.hex`.

### 3. Build & Flash ATmega32U4 Pro Micro (`firmware-promicro`)

Requires `avr-gcc`, `avr-libc`, `binutils-avr`, and the Rust `nightly` toolchain with `rust-src` (for `-Z build-std=core` on `avr-none`):

```sh
rustup toolchain install nightly --profile minimal --component rust-src

cd crates/firmware-promicro
cargo +nightly build --release

# Convert ELF to Intel HEX and flash with avrdude
avr-objcopy -O ihex -R .eeprom ../../target/avr-none/release/firmware-promicro.elf mumen-promicro.hex
avrdude -p atmega32u4 -c avr109 -P /dev/ttyACM0 -U flash:w:mumen-promicro.hex:i
```

---

## CI, Release Blobs, & Supply-Chain Security

GitHub Actions (`.github/workflows/ci.yml`) runs on every push, pull request, and semantic version tag (`vMAJOR.MINOR.PATCH`):
1. **Workflow Security Audit**: Static analysis via [`zizmor`](https://github.com/zizmorcore/zizmor-action) with SHA-pinned actions and least-privilege `GITHUB_TOKEN` permissions.
2. **Version & Ancestry Validation**: Single-sourced `[workspace.package] version` in root `Cargo.toml`. When a `vMAJOR.MINOR.PATCH` tag is pushed, CI verifies that:
   - the tag matches the semantic version pattern `^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$`,
   - the tag version matches `Cargo.toml`'s `[workspace.package] version`, and
   - the tagged commit is an ancestor of `main`.
3. **Formatting & Core Unit Tests**: `cargo fmt --all -- --check` and `cargo test -p mumen-core`.
4. **Multi-Target Firmware Builds & `.hex` Packaging**: Builds all 5 Teensy 4.0 layout features and both ATmega32U4 Pro Micro layout features in release mode, converts each ELF to a versioned Intel `.hex` image (`mumen-teensy40-${VERSION}-${layout}.hex` and `mumen-promicro-${VERSION}-${layout}.hex`) with `SHA256SUMS`, and uploads them as workflow artifacts.
5. **Automated GitHub Releases**: Pushing a verified `vMAJOR.MINOR.PATCH` tag on `main` automatically publishes a GitHub Release with all 7 pre-built `.hex` firmware blobs and `SHA256SUMS`.

---

## Related Projects & References

- **Loïc (*WydD*) Petit — [inputlag.science](https://inputlag.science/)** ([Controller Lag](https://inputlag.science/controller), [Methodology](https://inputlag.science/controller/methodology), [Results](https://inputlag.science/controller/results), and [`loic.petit/usblag`](https://gitlab.com/loic.petit/usblag)) — Primary research reference for the controller's prompt-detection debouncing, sub-500 µs internal sampling rate, synchronous USB endpoint priming, and `bInterval = 1` polling configuration.
- [`nulvox/UnlicProController`](https://github.com/nulvox/UnlicProController) — Original C / LUFA fight stick implementation for the ATmega32U4 Pro Micro.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
