use crate::audio::apu::OutputData;
use crate::audio::channels::channel::ChannelSignal;

pub struct NoiseGenerator {
    lfsr: u16,
    clocks: f32,
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self {
            lfsr: 0,
            clocks: 0.0,
        }
    }
}

impl NoiseGenerator {
    pub fn next_value(&mut self, sample_rate: f32, output_data: OutputData) -> f32 {
        if !output_data.is_on || !output_data.is_dac_on || !output_data.audio_unit_on {
            return 0.0;
        }

        let ChannelSignal::Noise {
            frequency,
            is_short,
            volume,
        } = output_data.signal
        else {
            return 0.0;
        };

        self.clocks += frequency / sample_rate;

        while self.clocks >= 1.0 {
            self.clocks -= 1.0;
            self.clock_lfsr(is_short);
        }

        let wave = if self.lfsr & 0b1 == 0 { 1.0 } else { -1.0 };

        wave * volume
    }

    fn clock_lfsr(&mut self, is_short: bool) {
        let new_bit = !(self.lfsr ^ (self.lfsr >> 1)) & 0b1;

        self.lfsr = (self.lfsr & !(1 << 15)) | (new_bit << 15);

        if is_short {
            self.lfsr = (self.lfsr & !(1 << 7)) | (new_bit << 7);
        }

        self.lfsr >>= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_cycles_through_32767_states_with_the_long_width() {
        let mut fixture = NoiseGenerator::default();
        let mut steps = 0;

        loop {
            fixture.clock_lfsr(false);
            steps += 1;

            if fixture.lfsr == 0 {
                break;
            }

            assert!(steps <= 32767);
        }

        assert_eq!(steps, 32767);
    }

    #[test]
    fn it_repeats_every_127_states_with_the_short_width() {
        let mut fixture = NoiseGenerator::default();

        // Lets the bits that were not set by the short feedback leave the register
        for _ in 0..32 {
            fixture.clock_lfsr(true);
        }

        let start = fixture.lfsr;

        for step in 1..=127 {
            fixture.clock_lfsr(true);

            assert_eq!(fixture.lfsr == start, step == 127, "step {step}");
        }
    }
}
