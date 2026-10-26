use crate::{Byte, Word};

#[derive(Default, Clone)]
#[readonly::make]
pub struct Dma {
    pub(crate) value: Byte,
    remaining_cycles: Byte,
}

impl Dma {
    pub fn single_step(&mut self) -> bool {
        if self.remaining_cycles == 0 {
            return false;
        }

        self.remaining_cycles = self.remaining_cycles.saturating_sub(1);

        if self.remaining_cycles == 0 {
            return true;
        }

        false
    }

    pub fn step(&mut self, cycles: u8) -> bool {
        if self.remaining_cycles == 0 {
            return false;
        }

        self.remaining_cycles = self.remaining_cycles.saturating_sub(cycles);

        if self.remaining_cycles == 0 {
            return true;
        }

        false
    }

    pub fn update(&mut self, value: Byte) {
        self.value = value;
        self.remaining_cycles = 160;
    }
}

impl From<&Dma> for Word {
    fn from(original: &Dma) -> Self {
        let mut value = original.value;

        if value > 0xDF {
            value &= 0xDF;
        }

        (value as Word) << 8 & 0xFF00
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_does_nothing_on_single_step_if_no_transfer_is_in_progress() {
        let mut fixture = Dma::default();

        assert!(!fixture.single_step());
        assert_eq!(fixture.remaining_cycles, 0);
    }

    #[test]
    fn it_finishes_the_transfer_after_160_single_steps() {
        let mut fixture = Dma::default();
        fixture.update(0xC0);

        for _ in 0..159 {
            assert!(!fixture.single_step());
        }

        assert!(fixture.single_step());
        assert!(!fixture.single_step());
    }

    #[test]
    fn it_restarts_the_transfer_when_updated_again() {
        let mut fixture = Dma::default();
        fixture.update(0xC0);

        for _ in 0..100 {
            fixture.single_step();
        }

        fixture.update(0xC1);

        assert_eq!(fixture.remaining_cycles, 160);
        assert_eq!(Word::from(&fixture), 0xC100);
    }

    #[test]
    fn it_finishes_the_transfer_equally_with_step_and_single_step() {
        let mut by_step = Dma::default();
        by_step.update(0xC0);

        assert!(!by_step.step(159));
        assert!(by_step.step(1));
    }
}
