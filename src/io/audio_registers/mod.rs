use crate::Byte;

pub mod nr52;
pub mod nrxx;

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

#[derive(Default, Clone)]
pub struct AudioRegWritten {
    pub control: bool,
    pub length: bool,
    pub sweep_or_wave_onoff: bool,
    pub envelope_or_wave_out_lvl: bool,
    pub frequency_or_poly_counter: bool,
    pub wave_pattern: bool,
}

impl AudioRegWritten {
    pub fn has_change(&self) -> bool {
        self.control
            || self.length
            || self.sweep_or_wave_onoff
            || self.envelope_or_wave_out_lvl
            || self.frequency_or_poly_counter
            || self.wave_pattern
    }
}
