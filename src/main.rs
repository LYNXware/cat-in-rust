#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc, IO};
use keyberon::matrix::Matrix;

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

    let mut pin43 = io.pins.gpio43.into_pull_up_input();
    // pin43.listen(Event::FallingEdge);
    // critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(pin43));

    // interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority2).unwrap();
     Matrix::new(
                [
                    io.pins.gpio3.into_pull_up_input().degrade(),
                    io.pins.gpio4.into_pull_up_input().degrade(),
                    io.pins.gpio5.into_pull_up_input().degrade(),
                    io.pins.gpio8.into_pull_up_input().degrade(),
                    io.pins.gpio9.into_pull_up_input().degrade(),
                ],
                [
                    io.pins.gpio0.into_push_pull_output().degrade(),
                    io.pins.gpio1.into_push_pull_output().degrade(),
                    io.pins.gpio2.into_push_pull_output().degrade(),
                    io.pins.gpio10.into_push_pull_output().degrade(),
                ],
            );

    #[allow(clippy::empty_loop)]
    loop {}
}
