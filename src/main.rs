#![warn(unused_imports)]
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::prelude::*;
use hal::{clock::ClockControl, peripherals::Peripherals, timer::TimerGroup, Rtc, IO};

mod board_modules;
use board_modules::left_finger;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();
    println!("Hello world!");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let (board_left_finger, io) = left_finger::BoardLeftFinger::new(io);

    #[allow(clippy::empty_loop)]
    loop {}
}
