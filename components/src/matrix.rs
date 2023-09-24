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
    pub const fn bit_len(&self) -> usize {
        InN::USIZE * OutN::USIZE
    }
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
    // TODO: this is currently bit_len, but it bytes, not bits :(
    type LEN = <InN as Mul<OutN>>::Output;
    fn read_state(&mut self, buf: &mut GenericArray<u8, Self::LEN>) {
        self.matrix.read_state(buf, &mut self.delayer)
    }
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
        let inner_buf = buf.as_mut_slice();
        let mut idx = 0;
        for byte in inner_buf.iter_mut() {
            // clear out previous state
            *byte = 0;
            for bit in 0..8 {
                // might be some bits left over
                if idx >= InN::to_usize() * OutN::to_usize() {
                    return;
                }

                // outer-loop
                let outputs_i = idx / InN::to_usize();
                // inner-loop
                let inputs_i = idx % InN::to_usize();

                // each outer-loop is for the `outputs_i`th output-pin to be in a low-state, before
                // returning back to a high-state
                if inputs_i == 0 {
                    let _todo_logerr = self.outs[outputs_i].set_low();
                    delayer.delay_us(5u16);
                }

                // the inner-loop is for each `inputs_i`th pin to test whether it is connected to
                // the low-state `outputs_i`th pin, thus indicating if the switch is closed
                let val = self.ins[inputs_i].is_low().unwrap_or_else(|_| panic!());
                if val {
                    let mask = 1 << bit;
                    *byte |= mask;
                }
                if inputs_i + 1 == InN::to_usize() {
                    let _todo_logerr = self.outs[outputs_i].set_high();
                }
                idx += 1;
            }
        }
    }
}
