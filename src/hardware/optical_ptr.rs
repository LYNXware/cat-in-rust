use core::convert::Infallible;
use embedded_hal::digital::v2::OutputPin;
use esp_backtrace as _;
use hal::{prelude::*, Delay};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
enum ADNSData {
    ProductId = 0x12, // should be 0x12,
    ProductId2 = 0x3e,
    RevisionId = 0x01,
    Cpi500v = 0x00,
    // TODO: 1 already assigned to revision id?
    // Cpi1000v =             0x01,
}
#[repr(u8)]
#[derive(IntoPrimitive, TryFromPrimitive)]
enum ADNSRegs {
    DeltaYReg = 0x03,
    DeltaXReg = 0x04,
    SqualReg = 0x05,
    ShutterUpperReg = 0x06,
    ShutterLowerReg = 0x07,
    MaximumPixelReg = 0x08,
    PixelSumReg = 0x09,
    MinimumPixelReg = 0x0A,
    PixelDataReg = 0x0B,
    Reset = 0x3a,
}
pub struct UninitADNS<'a> {
    pub sdio: &'a mut dyn OutputPin<Error = Infallible>,
    pub srl_clk: &'a mut dyn OutputPin<Error = Infallible>,
    pub not_reset: &'a mut dyn OutputPin<Error = Infallible>,
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
        res.write_reg(ADNSRegs::Reset, 0x5a, delay);
        delay.delay_us(50u16);
        let _ = res.not_reset.set_high();
        res
    }
}

struct ADNSDriver<'a> {
    sdio: &'a mut dyn OutputPin<Error = Infallible>,
    srl_clk: &'a mut dyn OutputPin<Error = Infallible>,
    not_reset: &'a mut dyn OutputPin<Error = Infallible>,
    not_chip_sel: &'a mut dyn OutputPin<Error = Infallible>,
    pix: [u8; 360],
}

impl<'a> ADNSDriver<'a> {
    fn sync(&mut self, delay: &mut Delay) {
        let _ = self.not_chip_sel.set_low();
        delay.delay_us(2u16);
        let _ = self.not_chip_sel.set_high();
    }

    fn write_reg(&mut self, addr: ADNSRegs, mut data: u8, delay: &mut Delay) {
        let mut addr: u8 = addr.into();
        let _ = self.not_chip_sel.set_low();
        for _ in 0..8 {
            let _ = self.srl_clk.set_low();
            delay.delay_us(1u16);
            if addr & 0b1000_0000 != 0 {
                let _ = self.sdio.set_high();
            } else {
                let _ = self.sdio.set_low();
            }
            addr <<= 1;
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

    fn read(&mut self, delay: &mut Delay) -> (i32, i32) {
        let y_sensor: i32 = self.read_reg(ADNSRegs::DeltaYReg, delay) as i32;
        let x_sensor: i32 = self.read_reg(ADNSRegs::DeltaXReg, delay) as i32;

        let dy = y_sensor * -1; // layouts_manager.mouse_factor[layer_control.active_layer][0] * -1
        let dx = x_sensor; //layouts_manager.mouse_factor[layer_control.active_layer][1];

        (dy, dx)

    }

    fn read_reg(&mut self, addr: ADNSRegs, delay: &mut Delay) -> u8 {
        let mut addr = addr as u8;
        for _ in 0..8 {
            let _ = self.not_chip_sel.set_low();
            let _ = self.srl_clk.set_low();
            if (addr  & 0x80u8) != 0 {
                let _ = self.sdio.set_high();
            } else {
                let _ = self.sdio.set_low();
            }

            addr <<= 1;
            delay.delay_us(1u16);
            let _ = self.srl_clk.set_high();
            delay.delay_us(1u16);
        }

        let mut res = 0u8;

        for i in 0..8 {
            let _ = self.srl_clk.set_low();

            // How to make the adns driver have a pin that is both output and input.
            if self.sdio.is_high() {
                res |= 0x01;
            }
            if i != 7 {
                res <<= 1;
            }
            let _ = self.srl_clk.set_high();
        }



        let mut bit_banged_read = 0;
        todo!("finish implementing")
    }
}
