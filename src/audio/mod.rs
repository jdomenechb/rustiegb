pub mod audio_unit_output;

use crate::memory::memory::Memory;
use crate::Word;
use audio_unit_output::AudioUnitOutput;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use std::sync::{Arc, RwLock};

pub enum VolumeEnvelopeDirection {
    UP,
    DOWN,
}

impl From<bool> for VolumeEnvelopeDirection {
    fn from(value: bool) -> Self {
        match value {
            false => Self::DOWN,
            true => Self::UP,
        }
    }
}

pub struct AudioUnit {
    auo: Box<dyn AudioUnitOutput>,
    memory: Arc<RwLock<Memory>>,
}

impl AudioUnit {
    pub fn new(au: Box<dyn AudioUnitOutput>, memory: Arc<RwLock<Memory>>) -> Self {
        Self { auo: au, memory }
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
            self.auo.stop_all();

            // FIXME: Might need to restart control registers
            return;
        }

        // TODO: sound 3-4

        // Sound 1
        if audio_triggers.0 {
            self.read_pulse(1, 0xFF14, 0xFF13, 0xFF12, 0xFF11, Some(0xFF10));
        }

        // Sound 2
        if audio_triggers.1 {
            self.read_pulse(2, 0xFF19, 0xFF18, 0xFF17, 0xFF16, None);
        }
    }

    fn read_pulse(
        &mut self,
        wave_n: u8,
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

        let frequency = ((control_reg as u16 & 0b111) << 8) | frequency_reg as u16;
        let frequency = 131072 as f32 / (2048 - frequency) as f32;

        let wave_duty = (length_reg >> 6) & 0b11;

        let wave_duty_percent: f32 = match wave_duty {
            0b00 => 0.125,
            0b01 => 0.25,
            0b10 => 0.50,
            0b11 => 0.75,
            _ => panic!("Invalid Wave Duty"),
        };

        let initial_envelope_vol = (volume_reg >> 4) & 0xF;
        let envelope_direction = VolumeEnvelopeDirection::from(volume_reg & 0b1000 == 0b1000);

        self.auo.play_pulse(
            wave_n,
            frequency,
            wave_duty_percent,
            initial_envelope_vol,
            envelope_direction,
        );
    }
}
