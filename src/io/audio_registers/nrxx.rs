use crate::{Byte, Word};

pub struct NRxxProperties {
    /// Enable the bits the register uses, so the unused can always be set to 1
    used_bits: Byte,
    /// Enable the bits that are write-only, so they can be turned to 1 when reading the register
    only_writable_bits: Byte,
    /// Enable the bits that need to be set to 0 on reset
    force_reset_on_bits: Byte,
}

impl NRxxProperties {
    const DEFAULT_USED_BITS: Byte = 0xFF;
    const DEFAULT_ONLY_WRITABLE_BITS: Byte = 0x00;
    const DEFAULT_FORCE_RESET_ON_BITS: Byte = 0x00;

    pub fn with_used_bits(&mut self, bits: Byte) -> &mut Self {
        self.used_bits = bits;
        self
    }

    pub fn with_only_writable_bits(&mut self, bits: Byte) -> &mut Self {
        self.only_writable_bits = bits;
        self
    }

    pub fn with_force_reset_on_bits(&mut self, bits: Byte) -> &mut Self {
        self.force_reset_on_bits = bits;
        self
    }
}

impl Default for NRxxProperties {
    fn default() -> Self {
        Self {
            used_bits: Self::DEFAULT_USED_BITS,
            only_writable_bits: Self::DEFAULT_ONLY_WRITABLE_BITS,
            force_reset_on_bits: Self::DEFAULT_FORCE_RESET_ON_BITS,
        }
    }
}

#[readonly::make]
pub struct NRxx {
    pub value: Byte,
    used_bits: Byte,
    only_writable_bits: Byte,
    force_reset_on_bits: Byte,
}

impl NRxx {
    pub fn new(default: Byte) -> Self {
        let properties = NRxxProperties::default();

        Self::new_from_properties(default, &properties)
    }

    pub fn new_from_properties(default: Byte, properties: &NRxxProperties) -> Self {
        Self {
            value: default,
            used_bits: properties.used_bits,
            only_writable_bits: properties.only_writable_bits,
            force_reset_on_bits: properties.force_reset_on_bits,
        }
    }

    pub fn read(&self) -> Byte {
        self.value | self.only_writable_bits
    }

    pub fn reset(&mut self) {
        self.value = !self.used_bits;
    }

    pub fn update(&mut self, value: Byte) {
        self.value = (value & self.used_bits) | !self.used_bits;
    }

    pub fn update_low_frequency(&mut self, frequency: Word) {
        self.value = (frequency & 0xFF) as Byte
    }

    pub fn update_high_frequency(&mut self, frequency: Word) {
        self.value = (self.value & 0b11111000) | ((frequency >> 8) & 0b111) as Byte
    }
}
