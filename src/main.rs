#![warn(unused_imports)]
#![no_std]
#![no_main]

use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
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
use keyberon::{key_code::KbHidReport, layout::Layout};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_hid::{
    descriptor::{KeyboardReport, SerializedDescriptor},
    hid_class::HIDClass,
};

mod board_modules;
#[allow(non_upper_case_globals)]
struct ButtonMatrix<'a, const InN: usize, const OutN: usize> {
    ins: [&'a dyn InputPin<Error = Infallible>; InN],
    outs: [&'a mut dyn OutputPin<Error = Infallible>; OutN],
}
#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> ButtonMatrix<'a, InN, OutN> {
    fn key_scan(&mut self, delay: &mut Delay) -> [[bool; InN]; OutN] {
        let mut res = [[false; InN]; OutN];
        for (out_dx, out_pin) in self.outs.iter_mut().enumerate() {
            let _todo_logerr = out_pin.set_low();
            delay.delay_us(5u32);
            for (in_dx, in_pin) in self.ins.iter().enumerate() {
                res[out_dx][in_dx] = in_pin.is_low().unwrap();
            }
            let _todo_logerr = out_pin.set_high();
        }
        res
    }
}

static mut USB_MEM: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Debug);
    log::trace!("entered main, logging initialized");
    let peripherals = Peripherals::take();
    log::trace!("Peripherals claimed");
    let mut system = peripherals.SYSTEM.split();
    log::trace!("System components claimed");
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    log::trace!("clocks claimed with boot-defaultes, and frozen");
    log::info!("Board resources claimed: peripherals, system, clocks");

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
    rtc.rwdt.disable();

    log::trace!("\t rtc.rwdt.disable()");
    wdt0.disable();
    log::trace!("\t wdt0.disable()");
    wdt1.disable();
    log::trace!("\t wdt1.disable()");
    log::info!("clocks configured: rtc-wdt, wdt0/1 disabled");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let uart_vdd_pin = io.pins.gpio1;
    let mut uart_vdd_pin = uart_vdd_pin.into_push_pull_output();
    uart_vdd_pin.set_high().unwrap();

    let t_r_pins = UartTxRx::new_tx_rx(io.pins.gpio44, io.pins.gpio43);
    // will log in the background. Don't need to use directly
    let mut _uart = Uart::new_with_config(
        peripherals.UART0,
        Some(UartConfig::default()),
        Some(t_r_pins),
        &clocks,
        &mut system.peripheral_clock_control,
    );

    log::info!("uart-setup: gnd: gnd, tx: 44, rx: 43, pwr: 1");

    let usb = USB::new(
        peripherals.USB0,
        io.pins.gpio18,
        io.pins.gpio19,
        io.pins.gpio20,
        &mut system.peripheral_clock_control,
    );

    let usb_bus = UsbBus::new(usb, unsafe { &mut USB_MEM });
    let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 1);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
        .manufacturer("shady-bastard")
        .product("totally not a malicious thing")
        .device_class(3)
        .build();

    let mut left_finger = ButtonMatrix {
        ins: [
            &io.pins.gpio47.into_pull_up_input(),
            &io.pins.gpio48.into_pull_up_input(),
            &io.pins.gpio45.into_pull_up_input(),
            &io.pins.gpio0.into_pull_up_input(),
        ],
        outs: [
            &mut io.pins.gpio38.into_push_pull_output(),
            &mut io.pins.gpio39.into_push_pull_output(),
            &mut io.pins.gpio40.into_push_pull_output(),
            &mut io.pins.gpio41.into_push_pull_output(),
            &mut io.pins.gpio42.into_push_pull_output(),
            &mut io.pins.gpio2.into_push_pull_output(),
        ],
    };
    for out in &mut left_finger.outs {
        out.set_high().unwrap();
    }
    let mut debouncer = keyberon::debounce::Debouncer::new([[false; 4]; 6], [[false; 4]; 6], 5);
    let mut layout = Layout::new(&board_modules::left_finger::LAYERS);

    let mut delay = Delay::new(&clocks);
    loop {
        delay.delay_ms(1u32);
        let report = left_finger.key_scan(&mut delay);
        let events = debouncer.events(report, Some(keyberon::debounce::transpose));
        for ev in events {
            layout.event(ev);
        }
        layout.tick();
        let report = layout.keycodes().collect::<KbHidReport>();
        let bytes = report.as_bytes();
        if bytes.iter().any(|&b| b != 0) {
            log::debug!("{:?}", report);
        }
        if !usb_dev.poll(&mut [&mut usb_hid]) {
            continue;
        }
        let res = usb_hid.push_raw_input(bytes);
        if res.is_err() {
            log::debug!("{:?}", res);
        }
    }
}
