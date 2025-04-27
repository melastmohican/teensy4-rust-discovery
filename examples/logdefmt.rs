//! Demonstrates a custom USB logging stack with defmt.
//!
//! As of version 0.5 of the BSP, the BSP doesn't include an internal
//! logging stack. You're responsible for sourcing or building your
//! own. This example shows one way to build your own. 
//!
//! # Overview
//!
//! The example sets up a USB serial device to stream data to the host. It
//! uses defmt-bbq as the defmt logger. Once the host configures the USB
//! device, the example periodically checks the logging queue for frames,
//! it and sends them to the host.
//!
//! Run on a Teensy 4.0 with `cargo run --example logdefmt`  
//! If the LED is blinking, the example should be running.
//! 
//! Check logs `cat /dev/cu.usbmodem141601 | defmt-print -e target/thumbv7em-none-eabihf/debug/examples/logdefmt`
//! You should see:
//! Hello from defmt! The count is 0
//! ERROR ERROR: 0
//! Hello from defmt! The count is 1
//! Hello from defmt! The count is 2
//! Hello from defmt! The count is 3
//! Hello from defmt! The count is 4

#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};
use defmt_bbq::DefmtConsumer;
use teensy4_bsp as bsp;
use teensy4_panic as _;

use bsp::{
    board,
    hal::{
        timer::Blocking,
        usbd::{
            gpt::{Instance::Gpt0, Mode},
            BusAdapter, EndpointMemory, EndpointState, Speed,
        },
    },
    interrupt,
    rt::{entry, interrupt},
};

use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbDeviceState, UsbVidPid},
};
use usbd_serial::SerialPort;

const SPEED: Speed = Speed::LowFull;
/// Looking for USB devices on your host? Search for this VID and PID.
/// lsusb | grep "5824:27dd"
/// Bus 020 Device 026: ID 5824:27dd
const VID_PID: UsbVidPid = UsbVidPid(0x5824, 0x27dd);
/// You could also look for this product ID.
const PRODUCT: &str = "teensy4-bsp-example";
const SERIAL_NUMBER: &str = "TEENSY4_LOG_123";

// Static variable to track USB configuration state
static USB_CONFIGURED: AtomicBool = AtomicBool::new(false);

// Global defmt consumer for USB interrupt access
static mut DEFMT_CONSUMER: Option<DefmtConsumer> = None;

// Global USB device and class for USB interrupt access
static mut USB_DEVICE: Option<UsbDevice<'static, BusAdapter>> = None;
static mut USB_CLASS: Option<SerialPort<'static, BusAdapter>> = None;
static mut USB_BUS: Option<UsbBusAllocator<BusAdapter>> = None;

static mut EP_MEMORY: EndpointMemory<1024> = EndpointMemory::new();
static mut EP_STATE: EndpointState = EndpointState::max_endpoints();

#[entry]
fn main() -> ! {
    // Initialize board resources
    let board::Resources {
        pit,
        mut pins,
        usb,
        mut gpio2,
        mut gpt1,
        ..
    } = board::t40(board::instances());
    let mut led = board::led(&mut gpio2, pins.p13);

    // Set up the USB bus with the components provided by the board

    let bus_adapter = BusAdapter::with_speed(
        usb,
        unsafe { &mut EP_MEMORY },
        unsafe { &mut EP_STATE },
        SPEED,
    );

    // We need USB interrupts to activate. Otherwise, we won't be able to respond
    // to the host.
    bus_adapter.set_interrupts(true);

    // We want periodic interrupts in order to check the defmt queue. The USB
    // device has its own periodic timers we can use for this purpose.
    bus_adapter.gpt_mut(Gpt0, |gpt| {
        gpt.stop();
        gpt.clear_elapsed();
        gpt.set_interrupt_enabled(true);
        gpt.set_mode(Mode::Repeat);
        gpt.set_load(10_000); // microseconds.
        gpt.reset();
        gpt.run();
    });

    unsafe {
        USB_BUS = Some(UsbBusAllocator::new(bus_adapter));
        USB_CLASS = Some(SerialPort::new(USB_BUS.as_ref().unwrap()));
        USB_DEVICE = Some(
            UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), VID_PID)
                .product(PRODUCT)
                .serial_number(SERIAL_NUMBER)
                .device_class(usbd_serial::USB_CLASS_CDC)
                .build(),
        );
        DEFMT_CONSUMER = Some(defmt_bbq::init().unwrap());
    }

    // Enable USB interrupt
    unsafe {
        cortex_m::peripheral::NVIC::unmask(interrupt::USB_OTG1);
    }

    let mut delay = Blocking::<_, { board::PERCLK_FREQUENCY }>::from_pit(pit.0);

    // If the LED turns on, we've made it past init
    led.set();

    let mut counter = 0u32;
    loop {
        led.toggle();
        delay.block_ms(250);

        defmt::println!("Hello from defmt! The count is {=u32}", counter);
        defmt::trace!("TRACE: {=u32}", counter);

        if counter % 3 == 0 {
            defmt::debug!("DEBUG: {=u32}", counter);
        }

        if counter % 5 == 0 {
            defmt::info!("INFO: {=u32}", counter);
        }

        if counter % 7 == 0 {
            defmt::warn!("WARN: {=u32}", counter);
        }

        if counter % 31 == 0 {
            defmt::error!("ERROR: {=u32}", counter);
        }

        counter = counter.wrapping_add(1);
    }
}

// USB interrupt handler
#[interrupt]
fn USB_OTG1() {
    // Get references to our global variables
    let (usb_device, usb_class, defmt_consumer) = unsafe {
        (
            USB_DEVICE.as_mut().unwrap(),
            USB_CLASS.as_mut().unwrap(),
            DEFMT_CONSUMER.as_mut().unwrap(),
        )
    };

    // If we're here because the timer elapsed, we should clear
    // that status.
    usb_device.bus().gpt_mut(Gpt0, |gpt| {
        while gpt.is_elapsed() {
            gpt.clear_elapsed();
        }
    });

    // Do we have USB packets to handle?
    if usb_device.poll(&mut [usb_class]) {
        // Are we newly configured?
        if usb_device.state() == UsbDeviceState::Configured {
            // Is this our first configuration? See the imxrt-usbd API
            // documentation for more information on this requirement.
            if !USB_CONFIGURED.load(Ordering::Relaxed) {
                usb_device.bus().configure();
            }
            USB_CONFIGURED.store(true, Ordering::Relaxed);
        } else {
            // We might have lost our configuration!
            USB_CONFIGURED.store(false, Ordering::Relaxed);
        }
    }

    // We can only touch the class once we're configured...
    if USB_CONFIGURED.load(Ordering::Relaxed) {
        // Remove bytes from the defmt queue and send them to the host.
        while let Ok(grant) = defmt_consumer.read() {
            if let Ok(written) = usb_class.write(&grant) {
                grant.release(written);
            } else {
                break;
            }
        }
    }
}
