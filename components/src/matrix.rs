use hal::{
    blocking::delay::DelayUs,
    digital::v2::{InputPin, OutputPin},
};


/// Pin container in uninitialized form
#[allow(non_upper_case_globals)]
pub struct UninitKeyPins<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize> {
    pub ins: [InP; InN],
    pub outs: [OutP; OutN],
}

/// Pin container in usable form
#[allow(non_upper_case_globals)]
struct InitedKeyPins<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize> {
    ins: [InP; InN],
    outs: [OutP; OutN],
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
    delayer: D,
}

#[allow(non_upper_case_globals)]
impl<InP: InputPin, OutP: OutputPin, const InN: usize, const OutN: usize, D: DelayUs<u16>>
    KeyDriver<InP, OutP, InN, OutN, D>
{
    pub fn new(
        matrix: UninitKeyPins<InP, OutP, InN, OutN>,
        delayer: D,
    ) -> Self {
        let matrix = matrix.init();
        Self {
            matrix,
            delayer,
        }
    }

    pub fn read_state(&mut self, buf: &mut [[bool ; OutN];InN]) {
        self.matrix.read_state(buf, &mut self.delayer)
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
    fn read_state(&mut self, buf: &mut [[bool ; OutN];InN], delayer: &mut impl DelayUs<u16>) {
        for (out_dx, out_pin) in self.outs.iter_mut().enumerate() {
            let _todo_logerr = out_pin.set_low();
            delayer.delay_us(5u16);
            for (in_dx, in_pin) in self.ins.iter().enumerate() {
                buf[out_dx][in_dx] = in_pin.is_low().unwrap_or_else(|_| panic!());
            }
            let _todo_logerr = out_pin.set_high();
        }
    }
}
