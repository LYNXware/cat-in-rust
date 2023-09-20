//! driver for the ADNS4080
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::spi::FullDuplex;
use fugit::{HertzU32, RateExtU32};
use hal::{
    gpio::{ OutputPin, InputPin },
    clock::Clocks,
    peripheral::Peripheral,
    spi::{FullDuplexMode, Instance, SpiMode},
    system::PeripheralClockControl,
    Delay, Spi,
};

const DY: u8 = 0x3;
const DX: u8 = 0x4;

struct SpiMouseSensor<'a, T: Instance> {
    spi: Spi<'a, T, FullDuplexMode>,
}

impl<'a, T: Instance> SpiMouseSensor<'a, T> {
    fn new<SCK: OutputPin, MOSI: OutputPin, MISO: InputPin, CS: OutputPin>(
        spi_periph: impl Peripheral<P = T> + 'a,
        sclk: impl Peripheral<P = SCK> + 'a,
        mosi: impl Peripheral<P = MOSI> + 'a,
        miso: impl Peripheral<P = MISO> + 'a,
        cs: impl Peripheral<P = CS> + 'a,
        peripheral_clock_control: &mut PeripheralClockControl,
        clocks: &Clocks,
    ) -> Self {
        let mut spi: Spi<'_, _, FullDuplexMode> = Spi::new(
            spi_periph,
            sclk,
            mosi,
            miso,
            cs,
            500u32.kHz(),
            SpiMode::Mode0,
            peripheral_clock_control,
            &clocks,
        );

        let mut delayer = Delay::new(&clocks);
        // sync with the mouse
        spi.send(0).unwrap();
        // reset the device
        spi.send(0).unwrap();
        let id = spi.read();
        assert_eq!(id.unwrap(), 12);
        delayer.delay_us(50u32);
        Self { spi }
    }
    fn read(&mut self) -> (u8, u8) {
            let dy = {
                self.spi.send(DY).expect("todo");
                self.spi.read().expect("todo")
            };
            let dx = {
                self.spi.send(DX).expect("todo");
                self.spi.read().expect("todo")
            };
            (dy, dx)
    }
}
