#![warn(unused_imports)]
#![no_std]
#![no_main]

use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use esp_println::logger::init_logger;
use esp_wifi::esp_now::{PeerInfo, ReceivedData};
use esp_wifi::{initialize, EspWifiInitFor};
use hal::clock::CpuClock;
use hal::otg_fs::{UsbBus, USB};
use hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    timer::TimerGroup,
    uart::{config::Config as UartConfig, TxRxPins as UartTxRx, Uart},
    Rtc, IO,
};
use hal::{prelude::*, Delay, Rng};
use keyberon::{key_code::KbHidReport, layout::Layout};
use usb_device::prelude::{UsbDeviceBuilder, UsbVidPid};
use usbd_hid::{
    descriptor::{KeyboardReport, SerializedDescriptor},
    hid_class::HIDClass,
};

mod board_modules;

static mut USB_MEM: [u32; 1024] = [0; 1024];
// TODO: set primary and secondary as config parameters
static PRIMARY_ADDR: [u8; 6] = [0x7c, 0xdf, 0xa1, 0xf4, 0x67, 0x38];
static SECONDARY_ADDR: [u8; 6] = [0x7c, 0xdf, 0xa1, 0xf5, 0x64, 0xb4];
// TODO: setup some sort of way to manage this in a sane way. This is honestly disgusting.
static IS_PRIMARY: bool = true;

#[entry]
fn main() -> ! {
    // panic!();
    init_logger(log::LevelFilter::Info);
    let peripherals = Peripherals::take();

    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.rwdt.disable();

    TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    )
    .wdt
    .disable();
    let mut timer = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    timer.wdt.disable();
    let timer = timer.timer0;

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let uart_vdd_pin = io.pins.gpio1;
    let mut uart_vdd_pin = uart_vdd_pin.into_push_pull_output();
    uart_vdd_pin.set_high().unwrap();

    let t_r_pins = UartTxRx::new_tx_rx(io.pins.gpio44, io.pins.gpio43);
    // // will log in the background. Don't need to use directly
    let mut _uart = Uart::new_with_config(
        peripherals.UART0,
        Some(UartConfig::default()),
        Some(t_r_pins),
        &clocks,
        &mut system.peripheral_clock_control,
    );
    log::info!("uart-setup: gnd: gnd, tx: 44, rx: 43, pwr: 1");

    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();
    log::info!("esp-init");

    let wifi = peripherals.RADIO.split().0;
    let mut esp_now = match esp_wifi::esp_now::EspNow::new(&init, wifi) {
        Ok(esp) => {
            log::info!("esp-made");
            esp
        }
        Err(e) => {
            log::error!("esp-creation error: {:?}", e);
            panic!();
        }
    };
    log::info!("esp-now version {}", esp_now.get_version().unwrap());

    let usb = USB::new(
        peripherals.USB0,
        io.pins.gpio18,
        io.pins.gpio19,
        io.pins.gpio20,
        &mut system.peripheral_clock_control,
    );
    log::info!("usb made");

    let usb_bus = UsbBus::new(usb, unsafe { &mut USB_MEM });
    log::info!("usb bus made");
    let mut usb_hid = HIDClass::new(&usb_bus, KeyboardReport::desc(), 1);
    log::info!("usb class made");

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0, 0))
        .manufacturer("shady-bastard")
        .product("totally not a malicious thing")
        .device_class(3)
        .build();
    log::info!("usb dev built");

    let ins: &[&dyn InputPin<Error = Infallible>; 4] = &[
        &io.pins.gpio21.into_pull_up_input(),
        &io.pins.gpio47.into_pull_up_input(),
        &io.pins.gpio48.into_pull_up_input(),
        &io.pins.gpio45.into_pull_up_input(),
    ];
    let outs: &mut [&mut dyn OutputPin<Error = Infallible>; 6] = &mut [
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
    let mut debouncer = keyberon::debounce::Debouncer::new([[false; 4]; 6], [[false; 4]; 6], 5);
    let mut layout = Layout::new(&board_modules::left_finger::LAYERS);

    let mut delay = Delay::new(&clocks);
    // TODO: clean this up (where is the vomit emoji when you need it...)
    if IS_PRIMARY {
        log::info!("adding secondary");
        let peer_add = esp_now.add_peer(PeerInfo {
            peer_address: SECONDARY_ADDR,
            lmk: None,
            channel: None,
            encrypt: false,
        });
        if peer_add.is_err() {
            log::error!("error adding secondary peer");
        } else {
            log::info!("added peer");
        }
    } else if esp_now
        .add_peer(PeerInfo {
            peer_address: PRIMARY_ADDR,
            lmk: None,
            channel: None,
            encrypt: false,
        })
        .is_err()
    {
        log::error!("bad pairing to primary");
    }

    loop {
        // TODO: clean this up (where is the vomit emoji when you need it...)
        if IS_PRIMARY {
            if !usb_dev.poll(&mut [&mut usb_hid]) {
                continue;
            }
            let r = esp_now.receive();
            if let Some(ReceivedData {
                len: _len,
                data,
                info: _info,
            }) = r
            {
                // todo: perform data-fusion in-house once we start working with multiple
                // switch-boards
                let data = &data[0..8];
                if data.iter().any(|&d| d != 0) {
                    log::info!("{:?}", data);
                }
                if let Err(e) = usb_hid.push_raw_input(data) {
                    // TODO: understand why pushing nil-state is a bad push.
                    log::error!("{:?}", e);
                    // match e {
                    //     usb_device::UsbError::WouldBlock => todo!(),
                    //     usb_device::UsbError::ParseError => todo!(),
                    //     usb_device::UsbError::BufferOverflow => todo!(),
                    //     usb_device::UsbError::EndpointOverflow => todo!(),
                    //     usb_device::UsbError::EndpointMemoryOverflow => todo!(),
                    //     usb_device::UsbError::InvalidEndpoint => todo!(),
                    //     usb_device::UsbError::Unsupported => todo!(),
                    //     usb_device::UsbError::InvalidState => todo!(),
                    // }
                }
            }
        } else {
            delay.delay_ms(1u32);
            let report = key_scan(&mut delay, ins, outs);
            let events = debouncer.events(report, Some(keyberon::debounce::transpose));
            // TODO send events and let the event data be fused up-stream before making the report?
            for ev in events {
                layout.event(ev);
            }
            layout.tick();
            let report = layout.keycodes().collect::<KbHidReport>();
            let bytes = report.as_bytes();
            log::info!("sending: {:?}", bytes);
            // let _ = esp_now.send(&PRIMARY_ADDR, bytes);
        }
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
