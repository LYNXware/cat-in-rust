use core::{convert::Infallible, num::{NonZeroU8, NonZeroI32}};
use embedded_hal::digital::v2::OutputPin;
use esp_backtrace as _;
use hal::{prelude::*, Delay};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use usbd_human_interface_device::device::mouse::WheelMouseReport;

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
enum ADNSData {
    ProductId = 0x12, // should be 0x12,
    ProductId2 = 0x3e,
    RevisionId = 0b1,
    DeltaYReg = 0b11,
    DeltaXReg = 0b100,
    SqualReg = 0b101,
    MaximumPixelReg = 0b1000,
    MinimumPixelReg = 0b1010,
    PixelSumReg = 0b1001,
    PixelDataReg = 0b1011,
    ShutterUpperReg = 0b110,
    ShutterLowerReg = 0b111,
    Reset = 0x3a,
    Cpi500v = 0x00,
    // TODO: 1 already assigned to revision id?
    // Cpi1000v =             0x01,
}
pub struct UninitADNS<'a> {
    pub sdio: &'a mut dyn OutputPin<Error = Infallible>,
    pub srl_clk: &'a mut dyn OutputPin<Error = Infallible>,
    pub not_reset: &'a dyn OutputPin<Error = Infallible>,
    pub not_chip_sel: &'a mut dyn OutputPin<Error = Infallible>,
}

impl<'a> UninitADNS<'a> {
    fn init(self, delay: &mut Delay) -> ADNSDriver<'a> {
        let mut res = ADNSDriver {
            sdio: self.sdio,
            srl_clk: self.srl_clk,
            not_reset: self.not_reset,
            not_chip_sel: self.not_chip_sel,
            pix: [0 ; 360],
        };

        res.sync(delay);
        res.write_reg(0x5a, ADNSData::Reset, delay);
        delay.delay_us(50u16);
        res.not_reset.set_high();
        res
    }
}

struct ADNSDriver<'a> {
    sdio: &'a mut dyn OutputPin<Error = Infallible>,
    srl_clk: &'a mut dyn OutputPin<Error = Infallible>,
    not_reset: &'a dyn OutputPin<Error = Infallible>,
    not_chip_sel: &'a mut dyn OutputPin<Error = Infallible>,
    pix: [u8; 360],
}

impl<'a> ADNSDriver<'a> {
    fn sync(&mut self, delay: &mut Delay) {
        let _ = self.not_chip_sel.set_low();
        delay.delay_us(2u16);
        let _ = self.not_chip_sel.set_high();
    }

    fn write_reg(&mut self, mut addr: ADNSData, data: u8, delay: &mut Delay) {
        let mut data: u8 = data.into();
        let _ = self.not_chip_sel.set_low();
        for _ in 0..8 {
            let _ = self.srl_clk.set_low();
            delay.delay_us(1u16);
            if addr as u8 & 0b1000_0000 != 0 {
                let _ = self.sdio.set_high();
            } else {
                let _ = self.sdio.set_low();
            }
            addr = ((addr as u8) << 1u8).try_into().unwrap();
            let _ = self.srl_clk.set_high();
            delay.delay_us(1u16);
        }

        for _ in 0..8 {
            let _ = self.srl_clk.set_low();
            delay.delay_us(1u16);
            if data & 0b1000_0000 != 0 {
                let _ = self.sdio.set_high();
            } else {
                let _ = self.sdio.set_low();
            }
            data <<= 1;
            let _ = self.srl_clk.set_high();
            delay.delay_us(1u16);
        }
        delay.delay_us(20u16);
        let _ = self.not_chip_sel.set_high();
    }

    fn read(&mut self, delay: &mut Delay) -> (Option<NonZeroI32>, Option<NonZeroI32>) {
        let y_sensor: Option<NonZeroI8> = self.read_reg(ADNSData::DeltaYReg, delay).into();
        let x_sensor: Option<NonZeroI8> = self.read_reg(ADNSData::DeltaXReg, delay).into();

        let dy = y_sensor.map(|dy|{ dy * -1/*layouts_manager.mouse_factor[layer_control.active_layer][0] * -1*/ });
        let dx = x_sensor.map(|dx| { dx * 1/*layouts_manager.mouse_factor[layer_control.active_layer][1]*/ } );

        (dy, dx)

    }

    fn read_reg(&mut self, mut addr: u8, delay: &mut Delay) -> bool {
        for _ in 0..8 {
            self.not_chip_sel.set_low();
            self.srl_clk.set_low();
            if addr & 0x80 != 0 {
                self.sdio.set_high();
            } else {
                self.sdio.set_low();
            }

            addr <<= 1;
            delay.delay_us(1u16);
            self.srl_clk.set_high();
            delay.delay_us(1u16);
        }

        let mut is_read = false;
    }
}
