use crate::memory::memory::Memory;
use crate::Word;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SupportedStreamConfig};
use std::sync::{Arc, RwLock};

pub trait AudioUnit {
    fn play_pulse(&self, frequency: f32);
}

pub struct CpalAudioUnit {
    device: Device,
}

impl CpalAudioUnit {
    pub fn new() -> Self {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .expect("failed to find a default output device");

        Self { device }
    }
}

impl AudioUnit for CpalAudioUnit {
    fn play_pulse(&self, frequency: f32) {
        let config = self.device.default_output_config().unwrap();

        match config.sample_format() {
            cpal::SampleFormat::F32 => run::<f32>(&self.device, &config.into(), frequency).unwrap(),
            cpal::SampleFormat::I16 => run::<i16>(&self.device, &config.into(), frequency).unwrap(),
            cpal::SampleFormat::U16 => run::<u16>(&self.device, &config.into(), frequency).unwrap(),
        }

        fn run<T>(
            device: &cpal::Device,
            config: &cpal::StreamConfig,
            frequency: f32,
        ) -> Result<(), anyhow::Error>
        where
            T: cpal::Sample,
        {
            let sample_rate = config.sample_rate.0 as f32;
            let channels = config.channels as usize;

            let mut sample_clock = 0f32;

            let mut next_value = move || {
                sample_clock = (sample_clock + 1.0) % sample_rate; // 0..44099

                -(sample_clock / (sample_rate / frequency / 2.0) % 2.0) + 0.5
            };

            let err_fn = |err| println!("an error occurred on stream: {}", err);

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

            stream.play()?;

            std::thread::sleep(std::time::Duration::from_millis(50));

            Ok(())
        }
    }
}

pub struct OutputDebugAudioUnit {}

impl AudioUnit for OutputDebugAudioUnit {
    fn play_pulse(&self, frequency: f32) {
        println!("Played at {} Hz", frequency);
    }
}

pub struct AudioUnitAdapter {
    au: Box<dyn AudioUnit>,
    memory: Arc<RwLock<Memory>>,
}

impl AudioUnitAdapter {
    pub fn new(au: Box<dyn AudioUnit>, memory: Arc<RwLock<Memory>>) -> Self {
        Self { au, memory }
    }

    pub fn step(&mut self) {
        let nr52;
        let audio_triggers;

        {
            let mut memory = self.memory.write().unwrap();

            nr52 = memory.read_byte(0xFF26);
            audio_triggers = memory.audio_has_been_trigerred();
        }

        let all_sound_trigger = nr52 & 0b10000000 == 0b10000000;

        if !all_sound_trigger {
            // FIXME: Maybe we need to stop sounds
            return;
        }

        // TODO: sound 2-4

        // Sound 1
        if audio_triggers.0 {
            self.read_pulse(0xFF14, 0xFF13, 0xFF12, 0xFF11, Some(0xFF10));
        }

        // Sound 2
        if audio_triggers.2 {
            self.read_pulse(0xFF19, 0xFF18, 0xFF17, 0xFF16, None);
        }
    }

    fn read_pulse(
        &mut self,
        control_addr: Word,
        frequency_addr: Word,
        volume_addr: Word,
        length_addr: Word,
        sweep_addr: Option<Word>,
    ) {
        let control_reg;
        let frequency_reg;
        let volume_reg;
        let length_reg;
        let sweep;

        {
            let memory = self.memory.read().unwrap();

            control_reg = memory.internally_read_byte(control_addr).unwrap();
            frequency_reg = memory.internally_read_byte(frequency_addr).unwrap();
            volume_reg = memory.internally_read_byte(volume_addr).unwrap();
            length_reg = memory.internally_read_byte(length_addr).unwrap();

            if sweep_addr.is_some() {
                sweep = Some(memory.internally_read_byte(sweep_addr.unwrap()));
            } else {
                sweep = None;
            }
        }

        let trigger = control_reg & 0b10000000 == 0b10000000;

        // TODO: Implement trigger

        let frequency = ((control_reg as u16 & 0b111) << 8) | frequency_reg as u16;
        let frequency = 131072 as f32 / (2048 - frequency) as f32;

        self.au.play_pulse(frequency);
    }
}
