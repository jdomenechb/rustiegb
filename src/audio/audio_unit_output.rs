use crate::audio::VolumeEnvelopeDirection;
use crate::Byte;
use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Stream};

pub trait AudioUnitOutput {
    fn play_pulse(&mut self, description: PulseDescription);
    fn stop_all(&mut self);
}

pub struct DebugAudioUnitOutput {}

impl AudioUnitOutput for DebugAudioUnitOutput {
    fn play_pulse(&mut self, description: PulseDescription) {
        println!(
            "S{}: Played at {} Hz, {}% duty. Env:  IV{}, D{}",
            description.pulse_n,
            description.frequency,
            description.wave_duty_percent * 100.0,
            description.initial_volume_envelope,
            match description.volume_envelope_direction {
                VolumeEnvelopeDirection::UP => "UP",
                VolumeEnvelopeDirection::DOWN => "DOWN",
            }
        );
    }

    fn stop_all(&mut self) {
        println!("Stopped all sounds");
    }
}

pub struct PulseDescription {
    pub pulse_n: u8,
    pub frequency: f32,
    pub wave_duty_percent: f32,
    pub initial_volume_envelope: Byte,
    pub volume_envelope_direction: VolumeEnvelopeDirection,
}

pub struct CpalAudioUnitOutput {
    device: Device,

    stream_1: Option<Stream>,
    stream_2: Option<Stream>,
    stream_3: Option<Stream>,
    stream_4: Option<Stream>,

    audio_status_1: Option<PulseDescription>,
    audio_status_2: Option<PulseDescription>,
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
            audio_status_1: None,
            audio_status_2: None,
        }
    }
}

impl AudioUnitOutput for CpalAudioUnitOutput {
    fn play_pulse(&mut self, description: PulseDescription) {
        let config = self.device.default_output_config().unwrap();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                run::<f32>(&self.device, &config.into(), &description).unwrap()
            }
            cpal::SampleFormat::I16 => {
                run::<i16>(&self.device, &config.into(), &description).unwrap()
            }
            cpal::SampleFormat::U16 => {
                run::<u16>(&self.device, &config.into(), &description).unwrap()
            }
        };

        match description.pulse_n {
            1 => self.stream_1 = Some(stream),
            2 => self.stream_2 = Some(stream),
            _ => panic!("Non pulse stream given"),
        }

        fn run<T>(
            device: &cpal::Device,
            config: &cpal::StreamConfig,
            description: &PulseDescription,
        ) -> Result<Stream, anyhow::Error>
        where
            T: cpal::Sample,
        {
            let sample_rate = config.sample_rate.0 as f32;
            let channels = config.channels as usize;

            let mut sample_clock = 0f32;
            let sample_in_period = sample_rate / description.frequency;
            let high_part_max = sample_in_period * description.wave_duty_percent;

            let volume_envelope = description.initial_volume_envelope;

            let mut next_value = move || {
                sample_clock = (sample_clock + 1.0) % sample_rate; // 0..44099

                let wave = if sample_clock % sample_in_period <= high_part_max {
                    1.0
                } else {
                    -1.0
                };

                let to_return = wave * volume_envelope as f32 / 0xF as f32;

                to_return
            };

            let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

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

    fn stop_all(&mut self) {
        self.stream_1 = None;
        self.stream_2 = None;
        self.stream_3 = None;
        self.stream_4 = None;
    }
}
