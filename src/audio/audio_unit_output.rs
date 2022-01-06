use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use parking_lot::RwLock;

use crate::audio::description::{NoiseDescription, PulseDescription, WaveDescription};
use crate::audio::sweep::Sweep;
use crate::audio::WaveOutputLevel;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Memory, Word};

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
    noise_description: Arc<RwLock<NoiseDescription>>,

    muted: bool,
}

impl CpalAudioUnitOutput {
    const MASTER_VOLUME: f32 = 0.25;

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
            noise_description: Arc::new(RwLock::new(NoiseDescription::default())),

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

        let next_value = move || {
            let sample_in_period;
            let high_part_max;
            let volume_envelope;
            let sample_clock;

            {
                let mut description = description.write();

                if !description.set || description.stop {
                    return 0.0;
                }

                sample_in_period = sample_rate / description.calculate_frequency();
                high_part_max = sample_in_period * description.wave_duty_percent;
                volume_envelope = description.volume_envelope.current_volume;
                sample_clock = description.next_sample_clock()
            }

            let wave = if sample_clock % sample_in_period <= high_part_max {
                1.0
            } else {
                0.0
            };

            (wave * volume_envelope as f32) / 7.5 - 1.0
        };

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value = next_value() * Self::MASTER_VOLUME;
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

        let next_value = move || {
            let sample_in_period;
            let output_level;
            let mut wave_sample;
            let duration_not_finished: f32;
            let sample_clock;

            {
                let mut description = description.write();

                if !description.set || !description.should_play {
                    return 0.0;
                }

                sample_clock = description.next_sample_clock();

                // How many samples are in one frequency oscillation
                sample_in_period = sample_rate / description.calculate_frequency();
                output_level = description.output_level;
                duration_not_finished =
                    if !description.use_length || description.remaining_steps > 0 {
                        1.0
                    } else {
                        0.0
                    };

                let current_wave_pos =
                    ((sample_clock % sample_in_period) / sample_in_period * 32.0).floor() as u8;

                wave_sample = description.wave.read_byte((current_wave_pos / 2) as Word);

                if current_wave_pos % 2 == 0 {
                    wave_sample >>= 4;
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

            (wave_sample / 0b1111) as f32 * duration_not_finished
        };

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value = next_value() * Self::MASTER_VOLUME;
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

    fn run_noise<T>(&mut self, config: &cpal::StreamConfig) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample,
    {
        let device = &self.device;
        let channels = config.channels as usize;

        let description = self.noise_description.clone();

        let next_value = move || {
            let lsfr;
            let volume_envelope;

            {
                let mut description = description.write();

                if !description.set || description.stop {
                    return 0.0;
                }

                lsfr = description.lfsr;
                volume_envelope = description.volume_envelope.current_volume;
            }

            let wave = (lsfr & 0b1) as f32;

            (wave * volume_envelope as f32) / 7.5 - 1.0
        };

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value = next_value() * Self::MASTER_VOLUME;
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

    pub fn play_pulse(&mut self, channel_n: u8, description: &PulseDescription) {
        if self.muted {
            return;
        }

        let stream;

        match channel_n {
            1 => {
                self.pulse_description_1.write().exchange(description);
                stream = &self.stream_1;
            }
            2 => {
                self.pulse_description_2.write().exchange(description);
                stream = &self.stream_2;
            }
            _ => panic!("Non pulse stream given"),
        }

        if stream.is_none() {
            let stream = match self.config.sample_format() {
                cpal::SampleFormat::F32 => self
                    .run_pulse::<f32>(&self.config.clone().into(), channel_n)
                    .unwrap(),
                cpal::SampleFormat::I16 => self
                    .run_pulse::<i16>(&self.config.clone().into(), channel_n)
                    .unwrap(),
                cpal::SampleFormat::U16 => self
                    .run_pulse::<u16>(&self.config.clone().into(), channel_n)
                    .unwrap(),
            };

            match channel_n {
                1 => self.stream_1 = Some(stream),
                2 => self.stream_2 = Some(stream),
                _ => panic!("Non pulse stream given"),
            }
        }
    }

    pub fn play_wave(&mut self, description: &WaveDescription) {
        if self.muted || !description.should_play {
            self.stream_3 = None;
            return;
        }

        self.wave_description.write().exchange(description);

        if self.stream_3.is_none() {
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

    pub fn play_noise(&mut self, description: &NoiseDescription) {
        if self.muted || description.stop {
            self.stream_4 = None;
            return;
        }

        self.noise_description.write().exchange(description);

        if self.stream_4.is_none() {
            let stream = match self.config.sample_format() {
                cpal::SampleFormat::F32 => {
                    self.run_noise::<f32>(&self.config.clone().into()).unwrap()
                }
                cpal::SampleFormat::I16 => {
                    self.run_noise::<i16>(&self.config.clone().into()).unwrap()
                }
                cpal::SampleFormat::U16 => {
                    self.run_noise::<u16>(&self.config.clone().into()).unwrap()
                }
            };

            self.stream_4 = Some(stream);
        }
    }

    pub fn stop_all(&mut self) {
        self.stream_1 = None;
        self.stream_2 = None;
        self.stream_3 = None;
        self.stream_4 = None;
    }

    pub fn set_mute(&mut self, muted: bool) {
        if self.muted != muted {
            self.stop_all();
            self.muted = muted;
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8) {
        self.noise_description.write().step(last_instruction_cycles);
    }

    pub fn step_64(&mut self) {
        self.pulse_description_1.write().step_64();
        self.pulse_description_2.write().step_64();
        self.noise_description.write().step_64();
    }

    pub fn step_128(&mut self, memory: Arc<RwLock<Memory>>) {
        self.pulse_description_1.write().step_128(memory);
    }

    pub fn step_256(&mut self) {
        self.pulse_description_1.write().step_256();
        self.pulse_description_2.write().step_256();
        self.wave_description.write().step_256();
        self.noise_description.write().step_256();
    }

    pub fn update(&mut self, memory: Arc<RwLock<Memory>>) {
        if self.pulse_description_1.read().stop {
            memory.write().set_audio_channel_inactive(1);
        }

        if self.pulse_description_2.read().stop {
            memory.write().set_audio_channel_inactive(2);
        }

        if !self.wave_description.read().should_play {
            memory.write().set_audio_channel_inactive(3);
        }

        if self.noise_description.read().stop {
            memory.write().set_audio_channel_inactive(4);
        }
    }

    pub fn reload_length(&mut self, channel_n: u8, pulse_length: Byte) {
        match channel_n {
            1 => self.pulse_description_1.write().reload_length(pulse_length),
            2 => self.pulse_description_2.write().reload_length(pulse_length),
            3 => self.wave_description.write().reload_length(pulse_length),
            4 => self.noise_description.write().reload_length(pulse_length),
            _ => panic!("Invalid channel provided"),
        }
    }

    pub fn reload_sweep(&mut self, sweep: Option<Sweep>) {
        self.pulse_description_1.write().reload_sweep(sweep);
    }
}
