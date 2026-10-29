mod noise;
mod pulse;
mod wave;

use crate::audio::apu::Apu;
use crate::audio::output::noise::NoiseGenerator;
use crate::audio::output::pulse::PulseGenerator;
use crate::audio::output::wave::WaveGenerator;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SizedSample, Stream, StreamConfig};
use std::sync::{Arc, RwLock};

pub struct CpalAudioOutput {
    stream: Stream,
    muted: bool,
}

impl CpalAudioOutput {
    const MASTER_VOLUME: f32 = 0.25;

    pub fn play(apu: Arc<RwLock<Apu>>) -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        let config = device.default_output_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::I8 => Self::run::<i8>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::I16 => Self::run::<i16>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::I32 => Self::run::<i32>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::I64 => Self::run::<i64>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::U8 => Self::run::<u8>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::U16 => Self::run::<u16>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::U32 => Self::run::<u32>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::U64 => Self::run::<u64>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::F32 => Self::run::<f32>(&device, config.into(), apu).unwrap(),
            cpal::SampleFormat::F64 => Self::run::<f64>(&device, config.into(), apu).unwrap(),
            _ => panic!("Invalid sample format"),
        };

        Self {
            stream,
            muted: false,
        }
    }

    fn run<T>(
        device: &Device,
        config: StreamConfig,
        apu: Arc<RwLock<Apu>>,
    ) -> Result<Stream, anyhow::Error>
    where
        T: SizedSample + FromSample<f32>,
    {
        let sample_rate = config.sample_rate as f32;
        let channels = config.channels as usize;

        let err_fn = |err| eprintln!("An error occurred on stream: {err}");

        let mut channel1 = PulseGenerator::default();
        let mut channel2 = PulseGenerator::default();
        let mut channel3 = WaveGenerator::default();
        let mut channel4 = NoiseGenerator::default();

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let apu = apu.read().unwrap();
                    let next_value1 = channel1.next_value(sample_rate, apu.get_output_data(1));
                    let next_value2 = channel2.next_value(sample_rate, apu.get_output_data(2));
                    let next_value3 = channel3.next_value(sample_rate, apu.get_output_data(3));
                    let next_value4 = channel4.next_value(sample_rate, apu.get_output_data(4));
                    let next_value = (next_value1 + next_value2 + next_value3 + next_value4) / 4.0;

                    let next_value = next_value * Self::MASTER_VOLUME;

                    let value: T = T::from_sample::<f32>(next_value);

                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;

        Ok(stream)
    }

    pub fn mute(&mut self) {
        if !self.muted {
            self.muted = true;
            self.stream.pause().unwrap();
        }
    }

    pub fn unmute(&mut self) {
        if self.muted {
            self.muted = false;
            self.stream.play().unwrap();
        }
    }
}
