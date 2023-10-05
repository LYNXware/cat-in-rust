use crate::ReadState;
use bitvec::{order::Lsb0, slice::BitSlice};
use core::ops::{Add, Div, Mul};
use hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};

use generic_array::{ArrayLength, GenericArray};
use typenum::{U7, U8};

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
    // first, get the raw size of the array
    InN: Mul<OutN>,
    // Now, we need to div-by-8 to get the number of bytes needed for all the bits...
    //
    // Without addinng 7, the last byte (if partially needed) is cut-out (4 * 5 -> 25. 25/8 -> 4,
    // losing the remainder).
    // This 7 ensures that we do not lose the remainder...
    // By dividing from the next multiple of 8, We solve that problem...
    // Consider an array of 8 by 8 = 64. Add the 7, it will be 71/8 = 64/8 under integer division...
    // BUT: an array of 6 by 7 = 42. div-by-8 has the 2-remainder lost.
    // Add 7, we have (42 + 7 = 49) / 8 = 6. The lost remainder is unwanted.
    <InN as Mul<OutN>>::Output: Add<U7>,
    // ...and we do the division
    <<InN as Mul<OutN>>::Output as Add<U7>>::Output: Div<U8>,
    // ...and tell the compiler it must impl array length. that becomes our LEN
    <<<InN as Mul<OutN>>::Output as Add<U7>>::Output as Div<U8>>::Output: ArrayLength,
    // Why not just use const-generics? That requires a nightly feature to do arithmetic in the
    // implementation constraints, and even so, the compiler warns that feature is incomplete, and
    // can cause panics
{
    // TODO: this is currently bit_len, but it bytes, not bits :(
    type LEN = <<<InN as Mul<OutN>>::Output as Add<U7>>::Output as Div<U8>>::Output;
    fn read_state(&mut self, buf: &mut GenericArray<u8, Self::LEN>) {
        self.matrix.read_state(buf, &mut self.delayer);
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
    fn read_state<LEN: ArrayLength>(
        &mut self,
        buf: &mut GenericArray<u8, LEN>,
        delayer: &mut impl DelayUs<u16>,
    ) {
        let inner_buf = buf.as_mut_slice();
        let bits = BitSlice::<_, Lsb0>::from_slice_mut(inner_buf);
        for (idx, mut bit) in bits.iter_mut().enumerate() {
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
            let is_set = self.ins[inputs_i].is_low().unwrap_or_else(|_| panic!());
            bit.set(is_set);
            if inputs_i + 1 == InN::to_usize() {
                let _todo_logerr = self.outs[outputs_i].set_high();
            }
        }
    }
}
