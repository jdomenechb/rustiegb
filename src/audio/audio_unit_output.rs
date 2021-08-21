use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use parking_lot::RwLock;

use crate::audio::description::{PulseDescription, WaveDescription};
use crate::audio::{VolumeEnvelopeDirection, WaveOutputLevel};
use crate::memory::memory_sector::ReadMemory;
use crate::Word;

pub trait AudioUnitOutput {
    fn play_pulse(&mut self, description: &PulseDescription);
    fn play_wave(&mut self, description: &WaveDescription);
    fn stop_all(&mut self);
    fn set_mute(&mut self, muted: bool);
    fn step_64(&mut self);
}

pub struct CpalAudioUnitOutput {
    device: Device,
    config: SupportedStreamConfig,

    stream_1: Option<Stream>,
    stream_2: Option<Stream>,
    stream_3: Option<Stream>,
    stream_4: Option<Stream>,

    pulse_description_1: Arc<RwLock<PulseDescription>>,
    pulse_description_2: Arc<RwLock<PulseDescription>>,
    wave_description: Arc<RwLock<WaveDescription>>,

    muted: bool,
}

impl CpalAudioUnitOutput {
    pub fn new() -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        let config = device.default_output_config().unwrap();

        Self {
            device,
            config,
            stream_1: None,
            stream_2: None,
            stream_3: None,
            stream_4: None,

            pulse_description_1: Arc::new(RwLock::new(PulseDescription::default())),
            pulse_description_2: Arc::new(RwLock::new(PulseDescription::default())),
            wave_description: Arc::new(RwLock::new(WaveDescription::default())),

            muted: false,
        }
    }

    fn run_pulse<T>(
        &mut self,
        config: &cpal::StreamConfig,
        pulse_n: u8,
    ) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample,
    {
        let device = &self.device;
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let description = match pulse_n {
            1 => self.pulse_description_1.clone(),
            2 => self.pulse_description_2.clone(),
            _ => panic!("Invalid pulse number"),
        };

        let mut sample_clock = 0f32;

        let mut next_value = move || {
            let sample_in_period;
            let high_part_max;
            let volume_envelope;

            {
                let description = description.read();
                sample_in_period = sample_rate / description.frequency;
                high_part_max = sample_in_period * description.wave_duty_percent;
                volume_envelope = description.volume_envelope;
            }

            sample_clock = (sample_clock + 1.0) % sample_rate; // 0..44099

            let wave = if sample_clock % sample_in_period <= high_part_max {
                1.0
            } else {
                -1.0
            };

            let to_return =
                wave * volume_envelope as f32 / 0xF as f32 * volume_envelope as f32 / 14.0;

            to_return
        };

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value = next_value();
                    let value: T = cpal::Sample::from::<f32>(&next_value);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
        )?;

        Ok(stream)
    }

    fn run_wave<T>(&mut self, config: &cpal::StreamConfig) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample,
    {
        let device = &self.device;
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let description = self.wave_description.clone();

        let mut sample_clock = 0f32;

        let mut next_value = move || {
            let sample_in_period;
            let output_level;
            let mut wave_sample;

            sample_clock = (sample_clock + 1.0) % sample_rate; // 0..44099

            {
                let description = description.read();

                // How many samples are in one frequency oscillation
                sample_in_period = sample_rate / description.frequency;

                output_level = description.output_level;

                let current_wave_pos =
                    ((sample_clock % sample_in_period) / sample_in_period * 32.0).floor() as u8;

                wave_sample = description.wave.read_byte((current_wave_pos / 2) as Word);

                if current_wave_pos % 2 == 0 {
                    wave_sample = wave_sample >> 4;
                } else {
                    wave_sample &= 0b1111;
                }
            }

            match output_level {
                WaveOutputLevel::Mute => wave_sample = 0,
                WaveOutputLevel::Vol50Percent => wave_sample >>= 1,
                WaveOutputLevel::Vol25Percent => wave_sample >>= 2,
                _ => {}
            }

            let to_return = (wave_sample / 0b1111) as f32;

            to_return
        };

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value = next_value();
                    let value: T = cpal::Sample::from::<f32>(&next_value);
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
        )?;

        Ok(stream)
    }
}

impl AudioUnitOutput for CpalAudioUnitOutput {
    fn play_pulse(&mut self, description: &PulseDescription) {
        if self.muted {
            return;
        }

        let stream;

        match description.pulse_n {
            1 => {
                let different = {
                    let read_pd = self.pulse_description_1.read();
                    *description != *read_pd
                };

                if different {
                    self.pulse_description_1.write().exchange(description);
                }

                stream = &self.stream_1;
            }
            2 => {
                let different = {
                    let read_pd = self.pulse_description_2.read();
                    *description != *read_pd
                };

                if different {
                    self.pulse_description_2.write().exchange(description);
                }

                stream = &self.stream_2;
            }
            _ => panic!("Non pulse stream given"),
        }

        if stream.is_none() {
            let stream = match self.config.sample_format() {
                cpal::SampleFormat::F32 => self
                    .run_pulse::<f32>(&self.config.clone().into(), description.pulse_n)
                    .unwrap(),
                cpal::SampleFormat::I16 => self
                    .run_pulse::<i16>(&self.config.clone().into(), description.pulse_n)
                    .unwrap(),
                cpal::SampleFormat::U16 => self
                    .run_pulse::<u16>(&self.config.clone().into(), description.pulse_n)
                    .unwrap(),
            };

            match description.pulse_n {
                1 => self.stream_1 = Some(stream),
                2 => self.stream_2 = Some(stream),
                _ => panic!("Non pulse stream given"),
            }
        }
    }

    fn play_wave(&mut self, description: &WaveDescription) {
        if self.muted {
            return;
        }

        let stream;
        let different;

        {
            let read_pd = self.wave_description.read();
            different = *description != *read_pd;
        }

        if different {
            self.wave_description.write().exchange(description);
        }

        stream = &self.stream_3;

        if stream.is_none() {
            let stream = match self.config.sample_format() {
                cpal::SampleFormat::F32 => {
                    self.run_wave::<f32>(&self.config.clone().into()).unwrap()
                }
                cpal::SampleFormat::I16 => {
                    self.run_wave::<i16>(&self.config.clone().into()).unwrap()
                }
                cpal::SampleFormat::U16 => {
                    self.run_wave::<u16>(&self.config.clone().into()).unwrap()
                }
            };

            self.stream_3 = Some(stream);
        }
    }

    fn stop_all(&mut self) {
        self.stream_1 = None;
        self.stream_2 = None;
        self.stream_3 = None;
        self.stream_4 = None;
    }

    fn set_mute(&mut self, muted: bool) {
        if self.muted != muted {
            self.stop_all();
            self.muted = muted;
        }
    }

    fn step_64(&mut self) {
        self.pulse_description_1.write().step_64();
        self.pulse_description_2.write().step_64();
    }
}
