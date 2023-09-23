use crate::ReadState;
use core::ops::Mul;
use hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

use generic_array::{ArrayLength, GenericArray};

/// Pin container in uninitialized form
#[allow(non_upper_case_globals)]
pub struct UninitKeyPins<InP: InputPin, OutP: OutputPin, InN: ArrayLength, OutN: ArrayLength> {
    pub ins: GenericArray<InP, InN>,
    pub outs: GenericArray<OutP, OutN>,
}

/// Pin container in usable form
#[allow(non_upper_case_globals)]
struct InitedKeyPins<InP: InputPin, OutP: OutputPin, InN: ArrayLength, OutN: ArrayLength> {
    ins: GenericArray<InP, InN>,
    outs: GenericArray<OutP, OutN>,
}

/// Contains initialized pins, and metadata for kb-matrix usage
#[allow(non_upper_case_globals)]
pub struct KeyDriver<
    InP: InputPin,
    OutP: OutputPin,
    InN: ArrayLength,
    OutN: ArrayLength,
    D: DelayUs<u16>,
> {
    matrix: InitedKeyPins<InP, OutP, InN, OutN>,
    delayer: D,
}

#[allow(non_upper_case_globals)]
impl<InP: InputPin, OutP: OutputPin, InN: ArrayLength, OutN: ArrayLength, D: DelayUs<u16>>
    KeyDriver<InP, OutP, InN, OutN, D>
{
    pub fn new(matrix: UninitKeyPins<InP, OutP, InN, OutN>, delayer: D) -> Self {
        let matrix = matrix.init();
        Self { matrix, delayer }
    }
}

impl<
        InP: InputPin,
        OutP: OutputPin,
        InN: ArrayLength + Mul<OutN>,
        OutN: ArrayLength,
        D: DelayUs<u16>,
    > ReadState for KeyDriver<InP, OutP, InN, OutN, D>
where
    <InN as Mul<OutN>>::Output: ArrayLength,
{
    type LEN = <InN as Mul<OutN>>::Output;
    fn read_state(&mut self, buf: &mut GenericArray<u8, Self::LEN>) {
        self.matrix.read_state(buf, &mut self.delayer)
    }

    // pub fn reset_with_new_tolerance(&mut self, n: u16) {
    //     self.debouncer = Debouncer::new([[false; InN]; OutN], [[false; InN]; OutN], n)
    // }
}

#[allow(non_upper_case_globals)]
impl<InP: InputPin, OutP: OutputPin, InN: ArrayLength, OutN: ArrayLength>
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
impl<InP: InputPin, OutP: OutputPin, InN: ArrayLength, OutN: ArrayLength>
    InitedKeyPins<InP, OutP, InN, OutN>
{
    /// The poll mechanism: For each output pins row/col, set to low. scan each input col/row to
    /// check if it follows the low-set
    /// TODO: constrain so that the there are 0..=7 extra bits.
    fn read_state<LEN: ArrayLength>(
        &mut self,
        buf: &mut GenericArray<u8, LEN>,
        delayer: &mut impl DelayUs<u16>,
    ) {
        let pin_finder = |byte, bit| {
            let idx = byte * 8 + bit;
            let out_pin = idx / OutN::to_usize();
            let in_pin = idx % InN::to_usize();
            (out_pin, in_pin)
        };
        let inner_buf = buf.as_mut_slice();
        for (n, byte) in inner_buf.iter_mut().enumerate() {
            for bit in 0..8 {
                let (out_pin, in_pin) = pin_finder(n, bit);
                if in_pin == 0 {
                    let _todo_logerr = self.outs[out_pin].set_low();
                    delayer.delay_us(5u16);
                }
                let val = self.ins[in_pin].is_low().unwrap_or_else(|_| panic!());
                let mask = (val as u8) << bit;
                *byte |= mask;
                if in_pin == 0 {
                    let _todo_logerr = self.outs[out_pin].set_high();
                }
            }
        }
    }
}
