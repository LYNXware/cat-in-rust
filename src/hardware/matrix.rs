use core::convert::Infallible;
use embedded_hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

use keyberon::debounce::Debouncer;

/// Pin container in uninitialized form
/// TODO: remove indirection
#[allow(non_upper_case_globals)]
pub struct UninitKeyPins<'a, const InN: usize, const OutN: usize> {
    pub ins: [&'a dyn InputPin<Error = Infallible>; InN],
    pub outs: [&'a mut dyn OutputPin<Error = Infallible>; OutN],
}

/// Pin container in usable form
/// TODO: remove indirection
#[allow(non_upper_case_globals)]
struct InitedKeyPins<'a, const InN: usize, const OutN: usize> {
    ins: [&'a dyn InputPin<Error = Infallible>; InN],
    outs: [&'a mut dyn OutputPin<Error = Infallible>; OutN],
}

/// Contains initialized pins, and metadata for kb-matrix usage
#[allow(non_upper_case_globals)]
pub struct KeyDriver<'a, const InN: usize, const OutN: usize, D: DelayUs<u16>> {
    matrix: InitedKeyPins<'a, InN, OutN>,
    pub debouncer: Debouncer<[[bool; InN]; OutN]>,
    delayer: D,
}

#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize, D: DelayUs<u16>> KeyDriver<'a, InN, OutN, D> {
    pub fn new(matrix: UninitKeyPins<'a, InN, OutN>, debounce_tolerance: u16, delayer: D) -> Self {
        let matrix = matrix.init();
        Self {
            matrix,
            debouncer: Self::gen_debouncer(debounce_tolerance),
            delayer,
        }
    }

    /// Provides a matrix of pressed-down keys
    pub fn key_scan(&mut self) -> [[bool; InN]; OutN] {
        self.matrix.key_scan(&mut self.delayer)
    }

    fn gen_debouncer(n: u16) -> Debouncer<[[bool; InN]; OutN]> {
        Debouncer::new([[false; InN]; OutN], [[false; InN]; OutN], n)
    }
}

#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> UninitKeyPins<'a, InN, OutN> {
    fn init(mut self) -> InitedKeyPins<'a, InN, OutN> {
        for out in &mut self.outs {
            let _ = out.set_high();
        }
        InitedKeyPins {
            ins: self.ins,
            outs: self.outs,
        }
    }
}

#[allow(non_upper_case_globals)]
impl<'a, const InN: usize, const OutN: usize> InitedKeyPins<'a, InN, OutN> {
    /// The poll mechanism: For each output pins row/col, set to low. scan each input col/row to
    /// check if it follows the low-set
    fn key_scan(&mut self, delayer: &mut impl DelayUs<u16>) -> [[bool; InN]; OutN] {
        let mut res = [[false; InN]; OutN];
        for (out_dx, out_pin) in self.outs.iter_mut().enumerate() {
            let _todo_logerr = out_pin.set_low();
            delayer.delay_us(5u16);
            for (in_dx, in_pin) in self.ins.iter().enumerate() {
                res[out_dx][in_dx] = in_pin.is_low().unwrap();
            }
            let _todo_logerr = out_pin.set_high();
        }
        res
    }
}
