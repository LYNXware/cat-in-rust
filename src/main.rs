#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::logger::init_logger;
use hal::otg_fs::{UsbBus, USB};
use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    timer::TimerGroup,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    Rtc, IO,
};
use hal::{prelude::*, Delay};

use keyberon::layout::Event;

mod board_modules;
use board_modules::left_finger;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor, MouseReport};
use usbd_hid::hid_class::HIDClass;

static mut USB_MEM: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Trace);
    log::trace!("entered main, logging initialized");
    // Obtain board resources
    let peripherals = Peripherals::take();
    log::trace!("Peripherals claimed");
    let mut system = peripherals.SYSTEM.split();
    log::trace!("System components claimed");
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::trace!("clocks claimed with boot-defaultes, and frozen");

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    log::trace!("Rtc claimed");
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::trace!("TimerGroup0 created");
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::trace!("TimerGroup1 created");
    let mut wdt1 = timer_group1.wdt;
    log::trace!("Disabling timers:");
    rtc.rwdt.disable();

    log::trace!("\t rtc.rwdt.disable()");
    wdt0.disable();
    log::trace!("\t wdt0.disable()");
    wdt1.disable();
    log::trace!("\t wdt1.disable()");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let uart_vdd_pin = io.pins.gpio1;
    let mut uart_vdd_pin = uart_vdd_pin.into_push_pull_output();
    uart_vdd_pin.set_high().unwrap();

    let t_r_pins = UartTxRx::new_tx_rx(io.pins.gpio5, io.pins.gpio6);
    let mut uart = Uart::new_with_config(
        peripherals.UART0,
        Some(UartConfig::default()),
        Some(t_r_pins),
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::info!("uart-setup");
    let mut delay = Delay::new(&clocks);

    let usb = USB::new(
        peripherals.USB0,
        io.pins.gpio18,
        io.pins.gpio19,
        io.pins.gpio20,
        &mut system.peripheral_clock_control,
    );
    let usb_bus = UsbBus::new(usb, unsafe { &mut USB_MEM });

    let mut hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 1);
    for _ in 0..5 {
        uart.write_bytes(b".");
        delay.delay_ms(500u32);
    }

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
        .manufacturer("shady-bastard")
        .product("totally not a malicious thing")
        .device_class(3)
        .build();

    let mut i = 2000;
    let down = &[0,0,0,0,0,0,0,0];
    let up = &[0,0,20,0,0,0,0,0];
    loop {
        if !usb_dev.poll(&mut [&mut hid]) {
            continue;
        }

        let rep = if i % 2000 < 1000 { down} else {up};
        i += 1;
        let _res = hid.push_raw_input(rep);
    }
}
