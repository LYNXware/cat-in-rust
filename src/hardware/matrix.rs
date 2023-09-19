//! Platform-agnostic implementation to init and read a keyboard matrix

use embedded_hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

use keyberon::debounce::Debouncer;

/// Pin container in uninitialized form
#[allow(non_upper_case_globals)]
pub struct UninitKeyPins<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize> {
    pub ins: [InP; InN],
    pub outs: [OutP; OutN],
}

/// Pin container in usable form
#[allow(non_upper_case_globals)]
struct InitedKeyPins<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize> {
    pub ins: [InP; InN],
    pub outs: [OutP; OutN],
}

/// Contains initialized pins, and metadata for kb-matrix usage
#[allow(non_upper_case_globals)]
pub struct KeyDriver<
    InP: InputPin,
    OutP: OutputPin,
    const InN: usize,
    const OutN: usize,
    D: DelayUs<u16>,
> {
    matrix: InitedKeyPins<InP, OutP, InN, OutN>,
    pub debouncer: Debouncer<[[bool; InN]; OutN]>,
    delayer: D,
}

#[allow(non_upper_case_globals)]
impl<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize, D: DelayUs<u16>>
    KeyDriver<InP, OutP, InN, OutN, D>
{
    pub fn new(
        matrix: UninitKeyPins<InP, OutP, InN, OutN>,
        debounce_tolerance: u16,
        delayer: D,
    ) -> Self {
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

    // pub fn reset_with_new_tolerance(&mut self, n: u16) {
    //     self.debouncer = Debouncer::new([[false; InN]; OutN], [[false; InN]; OutN], n)
    // }
}

#[allow(non_upper_case_globals)]
impl<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize>
    UninitKeyPins<InP, OutP, InN, OutN>
{
    fn init(mut self) -> InitedKeyPins<InP, OutP, InN, OutN> {
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
impl<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize>
    InitedKeyPins<InP, OutP, InN, OutN>
{
    /// The poll mechanism: For each output pins row/col, set to low. scan each input col/row to
    /// check if it follows the low-set
    fn key_scan(&mut self, delayer: &mut impl DelayUs<u16>) -> [[bool; InN]; OutN] {
        let mut res = [[false; InN]; OutN];
        for (out_dx, out_pin) in self.outs.iter_mut().enumerate() {
            let _todo_logerr = out_pin.set_low();
            delayer.delay_us(5u16);
            for (in_dx, in_pin) in self.ins.iter().enumerate() {
                res[out_dx][in_dx] = in_pin.is_low().unwrap_or_else(|_| panic!());
            }
            let _todo_logerr = out_pin.set_high();
        }
        res
    }
}
