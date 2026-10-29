use crate::audio::apu::OutputData;
use crate::audio::channels::channel::ChannelSignal;

#[derive(Default)]
pub struct PulseGenerator {
    sample_clock: f32,
}

impl PulseGenerator {
    pub fn next_value(&mut self, sample_rate: f32, output_data: OutputData) -> f32 {
        println!("{output_data:?}");

        if !output_data.is_on || !output_data.is_dac_on || !output_data.audio_unit_on {
            return 0.0;
        }

        let ChannelSignal::Pulse {
            frequency,
            wave_duty,
        } = output_data.signal
        else {
            return 0.0;
        };

        self.sample_clock += 1.0;
        println!("Sample Clock: {}", self.sample_clock);
        let samples_in_period = sample_rate / frequency;
        let mut high_part_max = samples_in_period * wave_duty;

        let low_part_return;
        let high_part_return;

        if wave_duty < 0.75 {
            high_part_max = samples_in_period - high_part_max;
            low_part_return = 0.0;
            high_part_return = 1.0;
        } else {
            low_part_return = 1.0;
            high_part_return = 0.0;
        };

        let wave = if self.sample_clock % samples_in_period <= high_part_max {
            low_part_return
        } else {
            high_part_return
        };

        wave - 1.0

        //wave * (volume_envelope as f32 / 7.5) - 1.0
    }
}
