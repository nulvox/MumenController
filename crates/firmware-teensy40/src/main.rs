//! Teensy 4.0 Nintendo Switch USB HID fight stick firmware.
//!
//! ## Low-Latency Design (informed by [inputlag.science](https://inputlag.science/controller))
//!
//! - **Prompt-Detection Debouncing (`0 µs` edge delay)**: Switch closures transition on the very
//!   first sample and lock out contact bounce for 4 ms (`20 * 200 µs`).
//! - **Sub-Millisecond Internal Sampling (`200 µs` / 5 kHz)**: Exceeds Loïc *WydD* Petit's
//!   $\le 500\,\mu\text{s}$ recommendation on [inputlag.science](https://inputlag.science/controller),
//!   bounding internal scan jitter to $[0, 0.2]\,\text{ms}$ (0.1 ms average).
//! - **Immediate USB IN Endpoint Priming**: Whenever `scan_inputs` detects a state change
//!   (`next_report != prev_report`), it immediately primes the USB IN endpoint if idle, and
//!   `USB_OTG1` (`usb1`) immediately re-primes the IN endpoint with the latest 200 µs sample
//!   as soon as the host consumes the previous packet.
//! - **`bInterval = 1`**: Requests the fastest legal USB polling interval under both Full-Speed
//!   (1 ms / 1000 Hz) and High-Speed ($125\,\mu\text{s}$ / 8000 Hz microframe) host schedulers.

#![no_std]
#![no_main]

use teensy4_panic as _;

mod pins;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use crate::pins::BoardInputs;
    use mumen_core::{
        ControllerEngine, SwitchReport, HORI_POKKEN_PID, HORI_VID, REPORT_DESCRIPTOR,
    };
    use rtic_monotonics::systick::{Systick, *};
    use teensy4_bsp::board;
    use teensy4_bsp::hal::usbd::{BusAdapter, EndpointMemory, EndpointState};
    use usb_device::{
        bus::UsbBusAllocator,
        device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
        UsbError,
    };
    use usbd_hid::hid_class::HIDClass;

    /// `bInterval = 1` requests 125 µs (8000 Hz) microframe polling on High-Speed xHCI hosts
    /// and 1 ms (1000 Hz) polling on hosts that interpret gamepad `bInterval` in Full-Speed ms.
    const USB_POLL_INTERVAL: u8 = 1;

    /// Endpoint buffer allocation for `imxrt-usbd`.
    static EP_MEMORY: EndpointMemory<1024> = EndpointMemory::new();
    static EP_STATE: EndpointState = EndpointState::max_endpoints();

    type Bus = BusAdapter;

    #[shared]
    struct Shared {
        /// Latest 8-byte HID input report produced by the 200 µs (5 kHz) scan task.
        report: SwitchReport,
        /// USB HID class shared between `scan_inputs` (for immediate edge priming) and `usb1`.
        hid: HIDClass<'static, Bus>,
        /// USB device shared with `usb1`.
        device: UsbDevice<'static, Bus>,
    }

    #[local]
    struct Local {
        inputs: BoardInputs,
        engine: ControllerEngine,
    }

    #[init(local = [bus: Option<UsbBusAllocator<Bus>> = None])]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            usb,
            pins,
            mut gpio1,
            mut gpio2,
            mut gpio4,
            adc1,
            ..
        } = board::t40(cx.device);

        // Configure physical Teensy 4.0 GPIO/ADC pads for the selected layout feature
        let inputs = BoardInputs::new(pins, &mut gpio1, &mut gpio2, &mut gpio4, adc1);

        // Start 10 kHz Systick monotonic (100 µs tick precision) for 200 µs (5 kHz) input scanning
        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        // Initialize the High-Speed USB bus and enable USB interrupts
        let bus = BusAdapter::new(usb, &EP_MEMORY, &EP_STATE);
        bus.set_interrupts(true);
        let bus = cx.local.bus.insert(UsbBusAllocator::new(bus));

        // Allocate the HID class with the 86-byte HORI Pokken / Switch report descriptor
        let hid = HIDClass::new(bus, &REPORT_DESCRIPTOR, USB_POLL_INTERVAL);

        // Build the USB device with licensed HORI Pokken VID/PID (0x0F0D:0x0092)
        let device = UsbDeviceBuilder::new(bus, UsbVidPid(HORI_VID, HORI_POKKEN_PID))
            .manufacturer("HORI CO.,LTD.")
            .product("POKKEN CONTROLLER")
            .serial_number("MUMEN-T40")
            .max_packet_size_0(64)
            .max_power(500)
            .build();

        let engine = ControllerEngine::default();
        let report = SwitchReport::neutral();

        scan_inputs::spawn().ok();

        (
            Shared {
                report,
                hid,
                device,
            },
            Local { inputs, engine },
        )
    }

    /// High-frequency 200 µs (5 kHz) input scan task.
    ///
    /// Samples and debounces all configured hardware switches (and 8-bit fast ADC axes if enabled),
    /// runs SOCD/mode/lock processing via `mumen-core`, updates the shared USB report, and
    /// immediately primes the USB IN endpoint whenever an input edge occurs.
    #[task(shared = [report, hid, device], local = [inputs, engine], priority = 1)]
    async fn scan_inputs(mut cx: scan_inputs::Context) {
        let mut prev_report = SwitchReport::neutral();

        loop {
            let logical = cx.local.inputs.sample();
            let next_report = cx.local.engine.update(&logical);
            let changed = next_report != prev_report;
            prev_report = next_report;

            cx.shared.report.lock(|report| {
                *report = next_report;
            });

            // If a button/direction changed and the IN endpoint is currently unprimed,
            // prime it immediately so the very next host IN token gets the new edge without waiting.
            if changed {
                let bytes = next_report.to_bytes();
                (&mut cx.shared.device, &mut cx.shared.hid).lock(|device, hid| {
                    if device.state() == UsbDeviceState::Configured {
                        let _ = hid.push_raw_input(&bytes);
                    }
                });
            }

            Systick::delay(200.micros()).await;
        }
    }

    /// Hardware USB OTG1 interrupt handler.
    #[task(
        binds = USB_OTG1,
        shared = [report, hid, device],
        local = [configured: bool = false],
        priority = 2
    )]
    fn usb1(cx: usb1::Context) {
        let configured = cx.local.configured;
        let usb1::SharedResources {
            mut report,
            mut hid,
            mut device,
            ..
        } = cx.shared;

        let latest_bytes = report.lock(|r| r.to_bytes());

        (&mut device, &mut hid).lock(|device, hid| {
            // Poll control & class endpoints
            let _ = device.poll(&mut [hid]);

            if device.state() == UsbDeviceState::Configured {
                // imxrt-usbd requirement: configure non-zero endpoints once after host sets configuration
                if !*configured {
                    device.bus().configure();
                    *configured = true;
                }

                // Drain any 8-byte output report sent by the Switch so the OUT endpoint never stalls
                let mut out_buf = [0u8; 8];
                let _ = hid.pull_raw_output(&mut out_buf);

                // Prime the IN endpoint with the freshest 200 µs sample as soon as the previous IN transfer completes
                match hid.push_raw_input(&latest_bytes) {
                    Ok(_) | Err(UsbError::WouldBlock) => {}
                    Err(_) => {}
                }
            } else {
                *configured = false;
            }
        });
    }
}
