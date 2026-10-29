use crate::audio::apu::OutputData;
use crate::audio::channels::channel::ChannelSignal;

#[derive(Default)]
pub struct WaveGenerator {
    phase: f32,
}

impl WaveGenerator {
    pub fn next_value(&mut self, sample_rate: f32, output_data: OutputData) -> f32 {
        if !output_data.is_on || !output_data.is_dac_on || !output_data.audio_unit_on {
            return 0.0;
        }

        let ChannelSignal::Wave {
            frequency,
            volume,
            samples,
        } = output_data.signal
        else {
            return 0.0;
        };

        self.phase = (self.phase + frequency / sample_rate) % 1.0;

        let sample = samples[(self.phase * samples.len() as f32) as usize % samples.len()];

        (sample as f32 / 7.5 - 1.0) * volume
    }
}
