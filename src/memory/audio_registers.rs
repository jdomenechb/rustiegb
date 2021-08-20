use crate::Byte;

#[readonly::make]
pub struct AudioRegisters {
    pub control: Byte,
    pub frequency: Byte,
    pub volume: Byte,
    pub length: Byte,
    pub sweep: Option<Byte>,
}

impl AudioRegisters {
    pub fn new(
        control: Byte,
        frequency: Byte,
        volume: Byte,
        length: Byte,
        sweep: Option<Byte>,
    ) -> Self {
        Self {
            control,
            frequency,
            volume,
            length,
            sweep,
        }
    }
}
