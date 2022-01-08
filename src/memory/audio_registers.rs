use crate::Byte;

#[readonly::make]
pub struct AudioRegisters {
    pub control: Byte,
    pub frequency_or_poly_counter: Byte,
    pub envelope_or_wave_out_lvl: Byte,
    pub length: Byte,
    pub sweep: Option<Byte>,
}

impl AudioRegisters {
    pub fn new(
        control: Byte,
        frequency: Byte,
        envelope: Byte,
        length: Byte,
        sweep: Option<Byte>,
    ) -> Self {
        Self {
            control,
            frequency_or_poly_counter: frequency,
            envelope_or_wave_out_lvl: envelope,
            length,
            sweep,
        }
    }
}
