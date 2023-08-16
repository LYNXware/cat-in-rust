use core::convert::Infallible;

use embedded_hal::digital::v2::{InputPin, OutputPin};
use esp_backtrace as _;
use hal::{prelude::*, Delay};
use keyberon::debounce::Debouncer;


#[allow(non_upper_case_globals)]
pub struct ButtonMatrix<'a, const InN: usize, const OutN: usize> {
    pub ins: [&'a dyn InputPin<Error = Infallible>; InN],
    pub outs: [&'a mut dyn OutputPin<Error = Infallible>; OutN],
}
#[allow(non_upper_case_globals)]
pub struct BoardModule<'a, const InN: usize, const OutN: usize> {
    pub matrix: ButtonMatrix<'a, InN, OutN>,
    pub debouncer: Debouncer<[[bool; InN]; OutN]>,
}
#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> BoardModule<'a, InN, OutN> {
    pub fn new(mut matrix: ButtonMatrix<'a, InN, OutN>, debounce_tolerance: u16) -> Self {
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
    pub fn key_scan(&mut self, delay: &mut Delay) -> [[bool; InN]; OutN] {
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
