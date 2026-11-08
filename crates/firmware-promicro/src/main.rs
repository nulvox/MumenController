//! SparkFun Pro Micro / Arduino Leonardo (ATmega32U4) Nintendo Switch USB HID fight stick firmware.
//!
//! ## Low-Latency Design (informed by [inputlag.science](https://inputlag.science/controller))
//!
//! - **1 ms (`1000 Hz`) Full-Speed USB Polling (`FS_POLL_INTERVAL_MS = 1`)**: Eliminates the
//!   0–4 ms (2.0 ms avg) USB polling penalty of a 4 ms `bInterval`.
//! - **Prompt-Detection Debouncing (`0 ms` edge delay)**: Immediate transition on the first
//!   1 ms USB SOF sample with a 4 ms (`4 * 1 ms`) contact bounce lockout window.
//! - **Synchronous USB-Coupled Sampling**: Samples the GPIO ports directly inside the USB
//!   SOF / endpoint interrupt right before pushing the 8-byte report into the IN FIFO,
//!   following Loïc *WydD* Petit's synchronous sampling recommendation on
//!   [inputlag.science](https://inputlag.science/controller).

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use panic_halt as _;

mod pins;

use arduino_hal::{entry, pac::PLL, pins, Peripherals};
use atmega_usbd::{SuspendNotifier, UsbBus};
use avr_device::{asm::sleep, interrupt};
use mumen_core::{ControllerEngine, DirectionalMode, HORI_POKKEN_PID, HORI_VID, REPORT_DESCRIPTOR};
use pins::ProMicroIo;
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
    UsbError,
};
use usbd_hid::hid_class::HIDClass;

/// Full-Speed USB polling interval in ms (`1` = 1 ms / 1000 Hz native polling rate).
const FS_POLL_INTERVAL_MS: u8 = 1;

static mut USB_BUS: Option<UsbBusAllocator<UsbBus<PLL>>> = None;
static mut USB_CTX: Option<UsbContext<PLL>> = None;

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let board_pins = pins!(dp);
    let pll = dp.PLL;
    let usb = dp.USB_DEVICE;

    let io = ProMicroIo::new(board_pins);

    // Configure 48 MHz USB PLL from the 16 MHz crystal:
    // 1. Prescale 16 MHz -> 8 MHz
    pll.pllcsr.write(|w| w.pindiv().set_bit());
    // 2. 96 MHz PLL output; /1.5 for 64 MHz high-speed timers, /2 for 48 MHz USB clock
    pll.pllfrq
        .write(|w| w.pdiv().mhz96().plltm().factor_15().pllusb().set_bit());
    // 3. Enable PLL and wait for lock
    pll.pllcsr.modify(|_, w| w.plle().set_bit());
    while pll.pllcsr.read().plock().bit_is_clear() {}

    let usb_bus = unsafe {
        let bus_ptr = &raw mut USB_BUS;
        (*bus_ptr).insert(UsbBus::with_suspend_notifier(usb, pll))
    };

    let hid = HIDClass::new(usb_bus, &REPORT_DESCRIPTOR, FS_POLL_INTERVAL_MS);

    let device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(HORI_VID, HORI_POKKEN_PID))
        .manufacturer("HORI CO.,LTD.")
        .product("POKKEN CONTROLLER")
        .serial_number("MUMEN-AVR")
        .max_packet_size_0(64)
        .max_power(500)
        .build();

    unsafe {
        let ctx_ptr = &raw mut USB_CTX;
        *ctx_ptr = Some(UsbContext {
            device,
            hid,
            io,
            engine: ControllerEngine::default(),
            last_mode: DirectionalMode::Dpad,
        });
    }

    unsafe { interrupt::enable() };

    loop {
        sleep();
    }
}

struct UsbContext<S: SuspendNotifier> {
    device: UsbDevice<'static, UsbBus<S>>,
    hid: HIDClass<'static, UsbBus<S>>,
    io: ProMicroIo,
    engine: ControllerEngine,
    last_mode: DirectionalMode,
}

impl<S: SuspendNotifier> UsbContext<S> {
    fn poll(&mut self) {
        // Poll USB control & class endpoints (fires on SOF every 1 ms as well as endpoint events)
        let _ = self.device.poll(&mut [&mut self.hid]);

        if self.device.state() == UsbDeviceState::Configured {
            // Drain any 8-byte output report sent by the Switch so the OUT endpoint never stalls
            let mut out_buf = [0u8; 8];
            let _ = self.hid.pull_raw_output(&mut out_buf);

            // Sample switches, run SOCD/mode/lock engine, and update mode LEDs if mode changed
            let inputs = self.io.sample();
            let report = self.engine.update(&inputs);
            if self.engine.mode != self.last_mode {
                self.last_mode = self.engine.mode;
                self.io.set_mode_leds(self.engine.mode);
            }

            let bytes = report.to_bytes();
            match self.hid.push_raw_input(&bytes) {
                Ok(_) | Err(UsbError::WouldBlock) => {}
                Err(_) => {}
            }
        }
    }
}

#[interrupt(atmega32u4)]
fn USB_GEN() {
    unsafe { poll_usb() };
}

#[interrupt(atmega32u4)]
fn USB_COM() {
    unsafe { poll_usb() };
}

/// # Safety
///
/// Must only be called from `USB_GEN` / `USB_COM` interrupt handlers with global interrupts disabled.
unsafe fn poll_usb() {
    let ctx_ptr = &raw mut USB_CTX;
    if let Some(ctx) = unsafe { (*ctx_ptr).as_mut() } {
        ctx.poll();
    }
}
