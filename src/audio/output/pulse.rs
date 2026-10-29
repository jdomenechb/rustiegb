use crate::audio::apu::OutputData;
use crate::audio::channels::channel::ChannelSignal;

#[derive(Default)]
pub struct PulseGenerator {
    phase: f32,
}

impl PulseGenerator {
    pub fn next_value(&mut self, sample_rate: f32, output_data: OutputData) -> f32 {
        if !output_data.is_on || !output_data.is_dac_on || !output_data.audio_unit_on {
            return 0.0;
        }

        let (ChannelSignal::Pulse {
            frequency,
            wave_duty,
            volume,
        }
        | ChannelSignal::Sweep {
            frequency,
            wave_duty,
            volume,
        }) = output_data.signal
        else {
            return 0.0;
        };

        self.phase = (self.phase + frequency / sample_rate) % 1.0;

        let is_high = if wave_duty < 0.75 {
            self.phase >= 1.0 - wave_duty
        } else {
            self.phase < wave_duty
        };

        let wave = if is_high { 1.0 } else { 0.0 };

        (wave * 2.0 - 1.0) * volume
    }
}
