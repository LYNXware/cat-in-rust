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
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode;
use keyberon::layout::Layout;
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};

use usbd_human_interface_device::device::mouse::{WheelMouse, WheelMouseReport};
use usbd_human_interface_device::device::{
    keyboard::{BootKeyboard, BootKeyboardConfig},
    mouse::WheelMouseConfig,
};
use usbd_human_interface_device::page::Keyboard as HidKeyboard;
use usbd_human_interface_device::prelude::*;

mod board_modules;
#[allow(non_upper_case_globals)]
struct ButtonMatrix<'a, const InN: usize, const OutN: usize> {
    ins: [&'a dyn InputPin<Error = Infallible>; InN],
    outs: [&'a mut dyn OutputPin<Error = Infallible>; OutN],
}
#[allow(non_upper_case_globals)]
struct BoardModule<'a, const InN: usize, const OutN: usize> {
    matrix: ButtonMatrix<'a, InN, OutN>,
    debouncer: Debouncer<[[bool; InN]; OutN]>,
}
#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> BoardModule<'a, InN, OutN> {
    fn new(mut matrix: ButtonMatrix<'a, InN, OutN>, debounce_tolerance: u16) -> Self {
        matrix.init();
        Self {
            matrix,
            debouncer: ButtonMatrix::gen_debouncer(debounce_tolerance),
        }
    }
}

#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> ButtonMatrix<'a, InN, OutN> {
    fn init(&mut self) {
        for out in &mut self.outs {
            let _ = out.set_high();
        }
    }
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
    fn gen_debouncer(n: u16) -> Debouncer<[[bool; InN]; OutN]> {
        Debouncer::new([[false; InN]; OutN], [[false; InN]; OutN], n)
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
    let mut classes = UsbHidClassBuilder::new()
        .add_device(WheelMouseConfig::default())
        .add_device(BootKeyboardConfig::default())
        .build(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
        .manufacturer("shady-bastard")
        .product("totally not a malicious thing")
        .device_class(3)
        .build();

    let left_finger = ButtonMatrix {
        ins: [
            &io.pins.gpio21.into_pull_up_input(),
            &io.pins.gpio47.into_pull_up_input(),
            &io.pins.gpio48.into_pull_up_input(),
            &io.pins.gpio45.into_pull_up_input(),
        ],
        outs: [
            &mut io.pins.gpio42.into_push_pull_output(),
            &mut io.pins.gpio41.into_push_pull_output(),
            &mut io.pins.gpio40.into_push_pull_output(),
            &mut io.pins.gpio39.into_push_pull_output(),
            &mut io.pins.gpio38.into_push_pull_output(),
            &mut io.pins.gpio37.into_push_pull_output(),
        ],
    };
    let mut encoder_a = io.pins.gpio35.into_pull_up_input();
    let mut encoder_b = io.pins.gpio36.into_pull_up_input();
    let mut wheel_gnd = io.pins.gpio0.into_push_pull_output();
    let _ = wheel_gnd.set_low();
    let mut wheel = WheelEncoder::new();

    let mut left_finger = BoardModule::new(left_finger, 5);

    let mut layout = Layout::new(&board_modules::left_finger::LAYERS);

    let mut delay = Delay::new(&clocks);
    loop {
        let scroll = wheel.read_encoder(&mut encoder_a, &mut encoder_b);
        delay.delay_us(300u32);
        let report = left_finger.matrix.key_scan(&mut delay);
        let events = left_finger
            .debouncer
            .events(report, Some(keyberon::debounce::transpose));
        for ev in events {
            layout.event(ev);
        }
        layout.tick();
        let ron_report = layout.keycodes();
        let hid_report = ron_report
            .map(|k: KeyCode| k as u8)
            .map(HidKeyboard::from);

        let keyboard = classes.device::<BootKeyboard<'_, _>, _>();
        match keyboard.write_report(hid_report) {
            Err(UsbHidError::WouldBlock | UsbHidError::Duplicate) | Ok(_) => {},
            Err(e) => {
                core::panic!("Failed to write keyboard report: {:?}", e)
            }
        };
        if let Some(scroll) = scroll {
            let scroll = match scroll {
                KeyCode::MediaScrollDown => 1,
                KeyCode::MediaScrollUp => -1,
                _ => panic!("this shouldn't happen"),
            };
            let mouse_report = WheelMouseReport {
                buttons: 0,
                x: 0,
                y: 0,
                vertical_wheel: scroll,
                horizontal_wheel: 0,
            };
            let mouse = classes.device::<WheelMouse<'_, _>, _>();
            match mouse.write_report(&mouse_report) {
                Err(UsbHidError::WouldBlock) | Ok(_) => {},
                Err(e) => {
                    core::panic!("Failed to write mouse report: {:?}", e)
                }
            };
        }
        usb_dev.poll(&mut [&mut classes]);
    }
}

/// TODO: wrap up the pins somehow
struct WheelEncoder {
    value: u8,
    state: bool,
    prev_state: bool,
    scroll_val: i8,
}

impl WheelEncoder {
    fn new() -> Self {
        Self {
            value: 0,
            state: true,
            prev_state: true,
            scroll_val: 0,
        }
    }
    fn read_encoder(
        &mut self,
        enc_a: &mut dyn InputPin<Error = Infallible>,
        enc_b: &mut dyn InputPin<Error = Infallible>,
    ) -> Option<KeyCode> {
        self.state = enc_a.is_high().unwrap();
        let res = if self.state == self.prev_state {
            None
        } else {
            let scroll = if enc_b.is_high().unwrap() == self.state {
                self.value -= 1;
                self.scroll_val = -1;
                KeyCode::MediaScrollDown
            } else {
                self.value += 1;
                self.scroll_val = 1;
                KeyCode::MediaScrollUp
            };
            Some(scroll)
        };
        self.prev_state = self.state;
        res
    }
}
