# MumenController Cleanup & Multi-Target Plan

## Goals

1. **Eliminate AI-generated slop and no-op stubs**: Remove all dead modules (`src/panic_handler.rs`, `src/panic/`, `src/spc.rs`), global heap allocation (`linked_list_allocator`), fake `read_pin` / `read_analog_pin` / `read_lock_pin` placeholders, and AI-hallucinated documentation (`docs/*.md`, `CHANGELOG.md`).
2. **Support both hardware targets cleanly**:
   - **Teensy 4.0** (`thumbv7em-none-eabihf`, Cortex-M7, `teensy4-bsp` + `imxrt-usbd` + `rtic`).
   - **SparkFun Pro Micro / Arduino Leonardo** (`avr-none` / `avr-atmega32u4`, ATmega32U4, `arduino-hal` + `avr-device` + `atmega-usbd`).
3. **Support multiple button/pinout layouts**:
   - Compile-time selectable button/pin layouts (e.g., full 20-input layout with lock & analog-toggle switches, 16-button digital layouts `d16` / `d16s` / `alternate`, and analog+digital `a4d17`).
4. **Zero personal crate forks**:
   - Use upstream crates (`usb-device`, `usbd-hid`, `teensy4-bsp`, `rtic`, `avr-device`, `arduino-hal`, `atmega-usbd`) with no dependencies on `nulvox/usbd-human-interface-device`, `nulvox/usb-device`, or `nulvox/avr-hal-atomicptr`.
5. **Host-testable core logic**:
   - Keep the HID report descriptor, 8-byte report packer, SOCD cleaner, directional mode logic (D-pad / Left Stick / Right Stick / Smash modifier), lock mask, and shift-register debouncer in a pure `no_std` library crate with zero MCU/HAL dependencies so `cargo test` runs on the host in CI.

---

## Target Repository Structure (Cargo Workspace)

Because Teensy 4.0 (ARM Cortex-M7, standard `stable` toolchain + `-Tt4link.x`) and ATmega32U4 (AVR, `nightly` `build-std = ["core"]` + `-mmcu=atmega32u4`) require different targets, linker flags, and HAL dependencies, a small Cargo workspace is much cleaner than cramming conflicting target flags into one crate:

```text
MumenController/
├── Cargo.toml                  # Workspace root
├── PLAN.md                     # This plan
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── crates/
│   ├── mumen-core/             # Pure no_std controller core (host-testable!)
│   │   ├── Cargo.toml          # Zero HAL deps; optional usbd-hid integration
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── hid.rs          # 86-byte HORI/Switch HID descriptor & 8-byte Report
│   │       ├── input.rs        # Logical buttons, D-pad, analog axes, Lock & Mode logic
│   │       ├── socd.rs         # Neutral, UpPriority, LastWin SOCD resolvers
│   │       ├── debounce.rs     # 1-byte shift-register debouncer per switch
│   │       └── layout.rs       # Layout definitions / traits (Standard, Alternate, D16, D16s, A4D17)
│   ├── firmware-teensy40/      # Teensy 4.0 binary crate
│   │   ├── .cargo/config.toml  # thumbv7em-none-eabihf + -C link-arg=-Tt4link.x
│   │   ├── Cargo.toml          # teensy4-bsp, rtic, usb-device, usbd-hid
│   │   └── src/
│   │       ├── main.rs         # RTIC app, single-owner USB polling, 1ms scan task
│   │       └── pins.rs         # Real iomuxc + gpio::Input / ADC pin wiring per layout feature
│   └── firmware-promicro/      # ATmega32U4 (Pro Micro / Leonardo) binary crate
│       ├── .cargo/config.toml  # avr-atmega32u4 target + build-std = ["core"]
│       ├── Cargo.toml          # arduino-hal, avr-device, atmega-usbd, usb-device, usbd-hid
│       └── src/
│           ├── main.rs         # AVR entry point, USB_GEN/USB_COM ISR or poll loop
│           └── pins.rs         # Real Pro Micro / Leonardo port pin wiring per layout feature
└── .github/workflows/ci.yml    # Runs host unit tests (mumen-core), Teensy build, and AVR build
```

---

## Execution Slices

### Slice 1: Delete Slop, Restore Licenses, and Establish `mumen-core` with Unit Tests
**Scope:**
1. Restore `LICENSE-MIT` and `LICENSE-APACHE` from git history (`e771313`).
2. Delete dead and hallucinated files:
   - `src/panic_handler.rs`
   - `src/panic/`
   - `src/spc.rs`
   - `memory.x` (broken `0x00000000` flash layout)
   - `CHANGELOG.md` and `docs/{DEBUG,QUICK_START,TECHNICAL,TESTING}.md`
3. Create `crates/mumen-core` containing the pure `no_std` controller engine:
   - **`hid.rs`**:
     - Exact 86-byte HORI Pokken / Switch wired controller HID report descriptor from `UnlicProController/Descriptors.c` (16 buttons, 4-bit hat + 4-bit constant, 4 analog axes `LX/LY/RX/RY`, 1 vendor byte, 8-byte output report) and `0x0F0D:0x0092` VID/PID constants.
     - 8-byte `SwitchReport` struct with `to_bytes() -> [u8; 8]`, button bitmasks (`Y, B, A, X, L, R, ZL, ZR, Minus, Plus, L3, R3, Home, Capture`), and hat encoding (`0..=7` directions, `0x08` / `0x0F` neutral).
   - **`socd.rs`**:
     - `SocdMode` (`Neutral`, `UpPriority`, `LastWin`, `LeftPriority`) operating on `DpadRaw { up, down, left, right }`.
   - **`debounce.rs`**:
     - Compact shift-register debouncer (`u8` history per switch, configurable sample mask).
   - **`input.rs`**:
     - `LogicalInputs` (buttons + directions + modifier/toggle switches: `lock`, `analog_left`, `analog_right`, `shift`).
     - Directional modes carried over from both the AVR (`Dpad`, `Analog`, `Smash` w/ `Shift` half-tilt `64`/`192`) and Teensy (`analog_left` / `analog_right` toggle switches) versions.
     - Menu button lock masking (`Start`/`Plus`, `Select`/`Minus`, `Home`, `Capture` suppressed when `lock` is active).
4. Write comprehensive unit tests (`cargo test -p mumen-core`) verifying report byte layout, button bit positions against `UnlicProController`, SOCD transitions, lock masking, and Smash/Analog stick modes.

**Verification:**
- `cargo test -p mumen-core` passes on the host (`x86_64-unknown-linux-gnu`).
- Zero heap (`alloc`), zero `unsafe` in `mumen-core`.

---

### Slice 2: Teensy 4.0 Firmware Crate (`crates/firmware-teensy40`) — Bootable Build & USB HID
**Scope:**
1. Move the Teensy 4.0 binary into `crates/firmware-teensy40` (with its own `.cargo/config.toml` passing `-C link-arg=-Tt4link.x` so `cargo build --release` links a valid Teensy 4.0 image with FCB/IVT headers).
2. Strip out `linked_list_allocator`, `imxrt-log`, `NullPoller`, and the duplicate `usb_device.poll()` calls.
3. Implement clean RTIC v2 USB handling using upstream `teensy4-bsp`, `usb-device`, and `usbd-hid`:
   - Configure `UsbDeviceBuilder` with `0x0F0D:0x0092` (`HORI CO., LTD.` / `POKKEN CONTROLLER` or `MUMEN PRO CONTROLLER`), `max_packet_size_0(64)`, `max_power(500)`.
   - Use `bInterval = 4` (which is $2^{4-1} \times 125\,\mu\text{s} = 1\,\text{ms}$ in USB 2.0 High-Speed on `imxrt-usbd`).
   - Call `device.bus().configure()` once when transitioning into `UsbDeviceState::Configured` (required by `imxrt-usbd` to enable non-zero endpoints).
   - Drain any host output reports via `hid.pull_raw_output()` so the OUT endpoint never stalls.
   - Keep `device` and `hid` owned exclusively by the `USB_OTG1` task (or cleanly locked), sharing only the 8-byte `SwitchReport` atomic/shared state with the 1 ms input scan task.

**Verification:**
- `cargo build --release` in `crates/firmware-teensy40` links cleanly into an ELF and converts via `rust-objcopy -O ihex` to a bootable `.hex`.

---

### Slice 3: Teensy 4.0 Real Hardware Pin Wiring & Multiple Button Layouts
**Scope:**
1. Restore real `teensy4_bsp::hal::iomuxc` pad configuration and `gpio::Input<Pxx>` hardware reads from `d98a54b` (replacing the `return false` stubs).
2. Implement compile-time layout selection via Cargo features in `crates/firmware-teensy40`:
   - `layout-standard` (default): Your full 20-pin Teensy 4.0 wiring (`P0` Lock w/ 100k pull-down; `P1`..`P19` buttons, D-pad, and `AnalogLeft`/`AnalogRight` toggles w/ 100k pull-ups).
   - `layout-alternate`: Your 16-pin Teensy 4.0 layout (`A`/`B` swapped on `P11`/`P14`, no `L2/R2/L3/R3/Lock/AnalogToggle`).
   - `layout-d16` / `layout-d16s`: Sequential pin layouts matching your `switchspec/d16.yml` (with Capture) and `switchspec/d16s.yml` (with Shift modifier for Smash/Analog/SOCD modes).
3. Wire the 1 ms RTIC `Systick` input scan loop:
   - Read real hardware pins → run `mumen-core` debouncer → run `mumen-core` SOCD + mode + lock processor → update shared `SwitchReport`.

**Verification:**
- `cargo build --release --no-default-features --features layout-...` succeeds for every layout feature on `thumbv7em-none-eabihf`.

---

### Slice 4: ATmega32U4 Pro Micro / Leonardo Firmware Crate (`crates/firmware-promicro`)
**Scope:**
1. Create `crates/firmware-promicro` targeting `avr-atmega32u4` (using modern upstream Rust AVR target support, `avr-device`, `arduino-hal`, and `agausmann/atmega-usbd` + `usb-device` + `usbd-hid`, without any `nulvox/*` forks).
2. Port your hardware pin mappings from `origin/hid:src/switches.rs` (`d3, a1, a0, sck, d5, a2, d0, miso, d10, mosi, d2, d7, d8, d6, d9` + Red/Blue mode indicator LEDs on `a3`/`d4`) and support layout feature flags (`layout-promicro-standard`, `layout-d16s`).
3. Feed the raw pin reads on the Pro Micro into the exact same `mumen-core` debouncer, mode switcher (`HOME + SHIFT` cycling `Dpad -> Analog -> Smash`), SOCD cleaner, and 8-byte `SwitchReport` used by the Teensy 4.0 crate.

**Verification:**
- `cargo build --release` in `crates/firmware-promicro` compiles and links the AVR `.elf` / `.hex` cleanly.

---

### Slice 5: CI Pipeline & Honest Documentation
**Scope:**
1. Update `.github/workflows/ci.yml` to run:
   - `cargo fmt --all -- --check`
   - `cargo test -p mumen-core` (host unit tests)
   - `cargo build --release` for `firmware-teensy40` across all layout features
   - `cargo build --release` for `firmware-promicro` across all layout features
2. Update `README.md` with:
   - Workspace overview (`mumen-core`, `firmware-teensy40`, `firmware-promicro`)
   - Pinout tables for Teensy 4.0 and Pro Micro across each layout feature
   - Build & flash commands (`cargo objcopy` + `teensy_loader_cli` / `avrdude`)
   - License section (`MIT OR Apache-2.0`).
