#![warn(unused_imports)]
#![no_std]
#![no_main]

use core::cell::RefCell;
use core::convert::Infallible;

use critical_section::Mutex;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use esp_println::logger::init_logger;
use hal::gpio::Unknown;
use hal::otg_fs::{UsbBus, USB};
use hal::{
    clock::ClockControl,
    gpio::{Gpio18, Gpio19, Gpio20},
    peripherals::Peripherals,
    timer::TimerGroup,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    Rtc, IO,
};
use hal::{prelude::*, Delay};
use keyberon::{key_code::KbHidReport, layout::Layout};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
use usbd_hid::{
    descriptor::{KeyboardReport, SerializedDescriptor},
    hid_class::HIDClass,
};

mod board_modules;

static mut USB_MEM: [u32; 1024] = [0; 1024];
static mut USB_BUS: Option<
    UsbBusAllocator<UsbBus<USB<Gpio18<Unknown>, Gpio19<Unknown>, Gpio20<Unknown>>>>,
> = None;
static mut USB_HID: Option<
    HIDClass<UsbBus<USB<Gpio18<Unknown>, Gpio19<Unknown>, Gpio20<Unknown>>>>,
> = None;
static mut USB_DEV: Option<
    UsbDevice<UsbBus<USB<Gpio18<Unknown>, Gpio19<Unknown>, Gpio20<Unknown>>>>,
> = None;

static KB_REPORT: Mutex<RefCell<Option<KbHidReport>>> = Mutex::new(RefCell::new(None));

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
    // will log in the background. don't need to invoke directly
    let mut _uart = Uart::new_with_config(
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

    // Normally, would set up Mutex<RefCell<Option<...>>>, but these are all
    // being accessed from within an interupt context (USB_DEVICE), which hasn't been activated yet
    unsafe {
        USB_BUS = Some(UsbBus::new(usb, &mut USB_MEM));
        USB_HID = Some(HIDClass::new(
            USB_BUS.as_ref().unwrap(),
            KeyboardReport::desc(),
            1,
        ));

        USB_DEV = Some(
            UsbDeviceBuilder::new(USB_BUS.as_ref().unwrap(), UsbVidPid(0, 0))
                .manufacturer("shady-bastard")
                .product("totally not a malicious thing")
                .device_class(3)
                .build(),
        );
    };

    let ins: &[&dyn InputPin<Error = Infallible>; 4] = &[
        &io.pins.gpio21.into_pull_up_input(),
        &io.pins.gpio47.into_pull_up_input(),
        &io.pins.gpio48.into_pull_up_input(),
        &io.pins.gpio45.into_pull_up_input(),
    ];
    let outs: &mut [&mut dyn OutputPin<Error = Infallible>; 6] = &mut [
        // pins.gpio21.into_push_pull_output(),
        &mut io.pins.gpio37.into_push_pull_output(),
        &mut io.pins.gpio38.into_push_pull_output(),
        &mut io.pins.gpio39.into_push_pull_output(),
        &mut io.pins.gpio40.into_push_pull_output(),
        &mut io.pins.gpio41.into_push_pull_output(),
        &mut io.pins.gpio42.into_push_pull_output(),
    ];
    for out in outs.iter_mut() {
        out.set_high().unwrap();
    }
    let mut debouncer = keyberon::debounce::Debouncer::new([[false; 4]; 6], [[false; 4]; 6], 2);
    let mut layout = Layout::new(&board_modules::left_finger::LAYERS);
    critical_section::with(|cs| {
        KB_REPORT.borrow_ref_mut(cs).replace(KbHidReport::default());
    });
    hal::interrupt::enable(
        hal::soc::peripherals::Interrupt::USB_DEVICE,
        hal::Priority::Priority2,
    ).unwrap();
    loop {
        let events = debouncer.events(
            key_scan(&mut delay, ins, outs),
            Some(keyberon::debounce::transpose),
        );
        for event in events {
            layout.event(event);
        }
        let report = layout.keycodes().collect::<KbHidReport>();
        critical_section::with(|cs| {
            KB_REPORT.borrow_ref_mut(cs).replace(report);
        });
    }
}

fn key_scan<const IN_N: usize, const OUT_N: usize>(
    delay: &mut Delay,
    ins: &[&dyn InputPin<Error = Infallible>; IN_N],
    outs: &mut [&mut dyn OutputPin<Error = Infallible>; OUT_N],
) -> [[bool; IN_N]; OUT_N] {
    let mut res = [[false; IN_N]; OUT_N];
    for (out_dx, out_pin) in outs.iter_mut().enumerate() {
        let _todo_logerr = out_pin.set_low();
        delay.delay_us(5u32);
        for (in_dx, in_pin) in ins.iter().enumerate() {
            res[out_dx][in_dx] = in_pin.is_low().unwrap();
        }
        let _todo_logerr = out_pin.set_high();
    }
    res
}

#[interrupt]
unsafe fn USB_DEVICE() {
    let dev_ref = USB_DEV.as_mut().unwrap();
    let hid_ref = USB_HID.as_mut().unwrap();

    if dev_ref.poll(&mut [hid_ref]) {
        usb_device::class_prelude::UsbClass::poll(hid_ref);
    }
    let _ = critical_section::with(|cs| {
        hid_ref.push_raw_input(KB_REPORT.borrow_ref(cs).as_ref().unwrap().as_bytes())
    });
}
