mod pulse;

use crate::audio::apu::Apu;
use crate::audio::output::pulse::PulseGenerator;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, FromSample, SizedSample, Stream, StreamConfig};
use std::sync::{Arc, RwLock};

pub trait ChannelGenerator {
    fn generate(&mut self, buffer: &mut [f32]);
}

pub struct CpalAudioOutput {
    stream: Stream,
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

        Self { stream }
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

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let apu = apu.read().unwrap();
                    let next_value1 = channel1.next_value(sample_rate, apu.get_output_data(1));
                    let next_value2 = channel2.next_value(sample_rate, apu.get_output_data(2));
                    let next_value = (next_value1 + next_value2) / 2.0;

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
}
