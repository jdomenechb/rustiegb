use crate::audio::{PulseDescription, VolumeEnvelopeDirection};
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream};
use std::sync::{Arc, RwLock};

pub trait AudioUnitOutput {
    fn play_pulse(&mut self, description: &PulseDescription);
    fn update_pulse(&mut self, description: &PulseDescription);
    fn stop_all(&mut self);
    fn toggle_mute(&mut self);
}

pub struct DebugAudioUnitOutput {}

impl AudioUnitOutput for DebugAudioUnitOutput {
    fn play_pulse(&mut self, description: &PulseDescription) {
        println!(
            "S{}: Played at {} Hz, {}% duty. Env: IV{}, {}, D{}/64",
            description.pulse_n,
            description.frequency,
            description.wave_duty_percent * 100.0,
            description.volume_envelope,
            match description.volume_envelope_direction {
                VolumeEnvelopeDirection::UP => "UP",
                VolumeEnvelopeDirection::DOWN => "DOWN",
            },
            description.remaining_volume_envelope_duration_in_1_64_s
        );

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    fn update_pulse(&mut self, description: &PulseDescription) {
        println!(
            "S{}: Updated at {} Hz, {}% duty. Env: IV{}, {}, D{}/64",
            description.pulse_n,
            description.frequency,
            description.wave_duty_percent * 100.0,
            description.volume_envelope,
            match description.volume_envelope_direction {
                VolumeEnvelopeDirection::UP => "UP",
                VolumeEnvelopeDirection::DOWN => "DOWN",
            },
            description.remaining_volume_envelope_duration_in_1_64_s
        );

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    fn stop_all(&mut self) {
        println!("Stopped all sounds");

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    fn toggle_mute(&mut self) {
        println!("Mute pressed");
    }
}

pub struct CpalAudioUnitOutput {
    device: Device,

    stream_1: Option<Stream>,
    stream_2: Option<Stream>,
    stream_3: Option<Stream>,
    stream_4: Option<Stream>,

    volume_envelope_1: Arc<RwLock<u8>>,
    volume_envelope_2: Arc<RwLock<u8>>,
    muted: bool,
}

impl CpalAudioUnitOutput {
    pub fn new() -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        Self {
            device,
            stream_1: None,
            stream_2: None,
            stream_3: None,
            stream_4: None,

            volume_envelope_1: Arc::new(RwLock::new(0)),
            volume_envelope_2: Arc::new(RwLock::new(0)),
            muted: false,
        }
    }

    fn run<T>(
        &mut self,
        config: &cpal::StreamConfig,
        description: &PulseDescription,
    ) -> Result<Stream, anyhow::Error>
    where
        T: cpal::Sample,
    {
        let device = &self.device;
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let mut sample_clock = 0f32;
        let sample_in_period = sample_rate / description.frequency;
        let high_part_max = sample_in_period * description.wave_duty_percent;

        let volume_envelope = match description.pulse_n {
            1 => {
                let mut content = self.volume_envelope_1.write().unwrap();
                *content = description.volume_envelope;

                self.volume_envelope_1.clone()
            }
            2 => {
                let mut content = self.volume_envelope_2.write().unwrap();
                *content = description.volume_envelope;

                self.volume_envelope_2.clone()
            }
            _ => panic!("Wrong pulse number"),
        };

        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate; // 0..44099

            let wave = if sample_clock % sample_in_period <= high_part_max {
                1.0
            } else {
                -1.0
            };

            let volume_envelope = volume_envelope.read().unwrap().clone();

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
}

impl AudioUnitOutput for CpalAudioUnitOutput {
    fn play_pulse(&mut self, description: &PulseDescription) {
        if self.muted {
            return;
        }

        let config = self.device.default_output_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.run::<f32>(&config.into(), description).unwrap(),
            cpal::SampleFormat::I16 => self.run::<i16>(&config.into(), description).unwrap(),
            cpal::SampleFormat::U16 => self.run::<u16>(&config.into(), description).unwrap(),
        };

        match description.pulse_n {
            1 => self.stream_1 = Some(stream),
            2 => self.stream_2 = Some(stream),
            _ => panic!("Non pulse stream given"),
        }
    }

    fn update_pulse(&mut self, description: &PulseDescription) {
        match description.pulse_n {
            1 => {
                let mut content = self.volume_envelope_1.write().unwrap();
                *content = description.volume_envelope;
            }
            2 => {
                let mut content = self.volume_envelope_2.write().unwrap();
                *content = description.volume_envelope;
            }
            _ => panic!("Non pulse stream given"),
        }
    }

    fn stop_all(&mut self) {
        self.stream_1 = None;
        self.stream_2 = None;
        self.stream_3 = None;
        self.stream_4 = None;
    }

    fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        self.stop_all();
    }
}
