use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, FromSample, Stream, StreamConfig, SupportedStreamConfig};
use parking_lot::RwLock;

use crate::audio::noise::NoiseDescription;
use crate::audio::pulse::PulseDescription;
use crate::audio::registers::{
    ControlRegisterUpdatable, EnvelopeRegisterUpdatable, FrequencyRegisterUpdatable,
    LengthRegisterUpdatable,
};
use crate::audio::wave::WaveDescription;
use crate::audio::wave::WaveOutputLevel;
use crate::io::registers::IORegisters;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::memory_sector::ReadMemory;
use crate::{Byte, Word};

pub struct CpalAudioUnitOutput {
    device: Device,
    config: SupportedStreamConfig,

    stream_mix: Option<Stream>,

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

        let mut description1 = PulseDescription::default();
        description1.init_sweep();

        let mut value = Self {
            device,
            config,

            stream_mix: None,

            pulse_description_1: Arc::new(RwLock::new(description1)),
            pulse_description_2: Arc::new(RwLock::new(PulseDescription::default())),
            wave_description: Arc::new(RwLock::new(WaveDescription::default())),
            noise_description: Arc::new(RwLock::new(NoiseDescription::default())),

            muted: false,
        };

        value.play();

        value
    }

    fn play(&mut self) {
        if self.muted {
            return;
        }

        if self.stream_mix.is_none() {
            let stream = match self.config.sample_format() {
                cpal::SampleFormat::I8 => self.run::<i8>().unwrap(),
                cpal::SampleFormat::I16 => self.run::<i16>().unwrap(),
                cpal::SampleFormat::I32 => self.run::<i32>().unwrap(),
                cpal::SampleFormat::I64 => self.run::<i64>().unwrap(),
                cpal::SampleFormat::U8 => self.run::<u8>().unwrap(),
                cpal::SampleFormat::U16 => self.run::<u16>().unwrap(),
                cpal::SampleFormat::U32 => self.run::<u32>().unwrap(),
                cpal::SampleFormat::U64 => self.run::<u64>().unwrap(),
                cpal::SampleFormat::F32 => self.run::<f32>().unwrap(),
                cpal::SampleFormat::F64 => self.run::<f64>().unwrap(),
                _ => panic!("Invalid sample format"),
            };

            self.stream_mix = Some(stream)
        }
    }

    fn run<T>(&mut self) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample + cpal::SizedSample + FromSample<f32>,
    {
        let config = &StreamConfig::from(self.config.clone());

        let device = &self.device;
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

        let description1 = self.pulse_description_1.clone();
        let description2 = self.pulse_description_2.clone();
        let description3 = self.wave_description.clone();
        let description4 = self.noise_description.clone();

        let pulse_func = CpalAudioUnitOutput::next_value_pulse;
        let wave_func = CpalAudioUnitOutput::next_value_wave;
        let noise_func = CpalAudioUnitOutput::next_value_noise;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let next_value1 =
                        pulse_func(description1.clone(), sample_rate) * Self::MASTER_VOLUME;
                    let next_value2 =
                        pulse_func(description2.clone(), sample_rate) * Self::MASTER_VOLUME;
                    let next_value3 =
                        wave_func(description3.clone(), sample_rate) * Self::MASTER_VOLUME;
                    let next_value4 =
                        noise_func(description4.clone(), sample_rate) * Self::MASTER_VOLUME;

                    let next_value = (next_value1 + next_value2 + next_value3 + next_value4) / 4.0;

                    let value: T = T::from_sample::<f32>(next_value);

                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    fn next_value_pulse(description: Arc<RwLock<PulseDescription>>, sample_rate: f32) -> f32 {
        let volume_envelope;
        let sample_clock;
        let wave_duty;
        let frequency;

        {
            let mut description = description.write();

            if description.stop {
                return 0.0;
            }

            sample_clock = description.next_sample_clock();
            volume_envelope = description.volume_envelope.current_volume;
            wave_duty = description.wave_duty.to_percent();
            frequency = description.calculate_frequency();
        }

        let sample_in_period = sample_rate / frequency;
        let mut high_part_max = sample_in_period * wave_duty;
        let low_part_return;
        let high_part_return;

        if wave_duty < 0.75 {
            high_part_max = sample_in_period - high_part_max;
            low_part_return = 0.0;
            high_part_return = 1.0;
        } else {
            low_part_return = 1.0;
            high_part_return = 0.0;
        };

        let wave = if sample_clock % sample_in_period <= high_part_max {
            low_part_return
        } else {
            high_part_return
        };

        wave * (volume_envelope as f32 / 7.5) - 1.0
    }

    fn next_value_wave(description: Arc<RwLock<WaveDescription>>, sample_rate: f32) -> f32 {
        let sample_in_period;
        let output_level;
        let mut wave_sample;
        let sample_clock;
        let frequency;
        let current_wave_pos;

        {
            let mut description = description.write();

            if !description.should_play || description.stop {
                return 0.0;
            }

            sample_clock = description.next_sample_clock();
            frequency = description.calculate_frequency();
            output_level = description.output_level;

            // How many samples are in one frequency oscillation
            sample_in_period = sample_rate / frequency;

            current_wave_pos =
                ((sample_clock % sample_in_period) / sample_in_period * 32.0).floor() as u8;

            wave_sample = description.wave.read_byte((current_wave_pos / 2) as Word);
        }

        if current_wave_pos % 2 == 0 {
            wave_sample >>= 4;
        } else {
            wave_sample &= 0b1111;
        }

        match output_level {
            WaveOutputLevel::Mute => wave_sample = 0,
            WaveOutputLevel::Vol50Percent => wave_sample >>= 1,
            WaveOutputLevel::Vol25Percent => wave_sample >>= 2,
            _ => {}
        }

        ((wave_sample as f32 / 16.0) - 0.5) * 2.0
    }

    fn next_value_noise(description: Arc<RwLock<NoiseDescription>>, sample_rate: f32) -> f32 {
        let sample_in_period;
        let volume_envelope;
        let sample_clock;
        let wave;

        {
            let mut description = description.write();

            if description.stop {
                return 0.0;
            }

            volume_envelope = description.volume_envelope.current_volume;

            sample_in_period = sample_rate / (description.calculate_frequency() * 8.0);
            sample_clock = description.next_sample_clock();

            if sample_clock % sample_in_period == 0.0 {
                description.update_lfsr();
            }

            wave = (!(description.lfsr & 0b1) & 0b1) as f32;
        }

        (wave * volume_envelope as f32) / 7.5 - 1.0
    }

    pub fn stop_all(&mut self) {
        self.stream_mix = None;
    }

    pub fn set_mute(&mut self, muted: bool) {
        if self.muted != muted {
            self.stop_all();
            self.muted = muted;
        }
    }

    pub fn step_64(&mut self) {
        self.pulse_description_1.write().step_64();
        self.pulse_description_2.write().step_64();
        self.noise_description.write().step_64();
    }

    pub fn step_128(&mut self, io_registers: Arc<RwLock<IORegisters>>) {
        self.pulse_description_1.write().step_128(io_registers);
    }

    pub fn step_256(&mut self) {
        self.pulse_description_1.write().step_256();
        self.pulse_description_2.write().step_256();
        self.wave_description.write().step_256();
        self.noise_description.write().step_256();
    }

    pub fn update(&mut self, io_registers: Arc<RwLock<IORegisters>>) {
        let mut io_registers = io_registers.write();

        if self.pulse_description_1.read().stop {
            io_registers.nr52.set_ro_channel_flag_inactive(1);
        }

        {
            if self.pulse_description_2.read().stop {
                io_registers.nr52.set_ro_channel_flag_inactive(2);
            }
        }

        {
            if self.wave_description.read().stop {
                io_registers.nr52.set_ro_channel_flag_inactive(3);
            }
        }

        {
            if self.noise_description.read().stop {
                io_registers.nr52.set_ro_channel_flag_inactive(4);
            }
        }
    }

    pub fn update_length(&mut self, channel_n: Byte, register: Byte) {
        match channel_n {
            1 => {
                self.pulse_description_1
                    .write()
                    .trigger_length_register_update(register);
            }

            2 => {
                self.pulse_description_2
                    .write()
                    .trigger_length_register_update(register);
            }

            3 => {
                self.wave_description
                    .write()
                    .trigger_length_register_update(register);
            }

            4 => {
                self.noise_description
                    .write()
                    .trigger_length_register_update(register);
            }

            _ => panic!("Invalid channel number"),
        }
    }

    pub fn update_sweep(&mut self, sweep: Byte) {
        self.pulse_description_1.write().reload_sweep(sweep);
    }

    pub fn update_control(
        &mut self,
        channel_n: Byte,
        register: Byte,
        next_frame_step_is_length: bool,
    ) {
        match channel_n {
            1 => {
                self.pulse_description_1
                    .write()
                    .trigger_control_register_update(register, next_frame_step_is_length);
            }

            2 => {
                self.pulse_description_2
                    .write()
                    .trigger_control_register_update(register, next_frame_step_is_length);
            }

            3 => {
                self.wave_description
                    .write()
                    .trigger_control_register_update(register, next_frame_step_is_length);
            }

            4 => {
                self.noise_description
                    .write()
                    .trigger_control_register_update(register, next_frame_step_is_length);
            }

            _ => panic!("Invalid channel number"),
        }
    }

    pub fn update_envelope(&mut self, channel_n: Byte, register: Byte) {
        match channel_n {
            1 => {
                self.pulse_description_1
                    .write()
                    .trigger_envelope_register_update(register);
            }

            2 => {
                self.pulse_description_2
                    .write()
                    .trigger_envelope_register_update(register);
            }

            4 => {
                self.noise_description
                    .write()
                    .trigger_envelope_register_update(register);
            }

            _ => panic!("Invalid channel provided"),
        }
    }

    pub fn update_frequency(&mut self, channel_n: Byte, register: Byte) {
        match channel_n {
            1 => {
                self.pulse_description_1
                    .write()
                    .trigger_frequency_register_update(register);
            }

            2 => {
                self.pulse_description_2
                    .write()
                    .trigger_frequency_register_update(register);
            }

            3 => {
                self.wave_description
                    .write()
                    .trigger_frequency_register_update(register);
            }

            _ => panic!("Invalid channel provided"),
        }
    }

    pub fn update_wave_onoff(&mut self, register: Byte) {
        self.wave_description
            .write()
            .trigger_wave_onoff_register_update(register);
    }

    pub fn update_wave_output_level(&mut self, register: Byte) {
        self.wave_description
            .write()
            .trigger_wave_output_level_register_update(register);
    }

    pub fn update_wave_pattern(&mut self, pattern: WavePatternRam) {
        self.wave_description
            .write()
            .trigger_wave_pattern_update(pattern);
    }

    pub fn update_noise_poly_counter(&mut self, register: Byte) {
        self.noise_description
            .write()
            .trigger_poly_counter_register_update(register);
    }
}
