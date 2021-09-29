use std::sync::Arc;

use parking_lot::RwLock;

use audio_unit_output::AudioUnitOutput;
use description::{PulseDescription, WaveDescription};

use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::memory::Memory;

pub mod audio_unit_output;
mod description;
pub mod sweep;

const CYCLES_1_256_SEC: u16 = 16384;
const CYCLES_1_128_SEC: u16 = 16384 * 2;
const CYCLES_1_64_SEC: u32 = 16384 * 4;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum VolumeEnvelopeDirection {
    Up,
    Down,
}

impl From<bool> for VolumeEnvelopeDirection {
    fn from(value: bool) -> Self {
        match value {
            false => Self::Down,
            true => Self::Up,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum WaveOutputLevel {
    Mute,
    Vol100Percent,
    Vol50Percent,
    Vol25Percent,
}

impl Into<f32> for WaveOutputLevel {
    fn into(self) -> f32 {
        match self {
            WaveOutputLevel::Mute => 0.0,
            WaveOutputLevel::Vol25Percent => 0.25,
            WaveOutputLevel::Vol50Percent => 0.5,
            WaveOutputLevel::Vol100Percent => 1.0,
        }
    }
}

pub struct AudioUnit {
    auo: Box<dyn AudioUnitOutput>,
    memory: Arc<RwLock<Memory>>,

    cycle_count: u16,
    cycle_128_count: u16,
    cycle_64_count: u32,
}

impl AudioUnit {
    pub fn new(au: Box<dyn AudioUnitOutput>, memory: Arc<RwLock<Memory>>) -> Self {
        Self {
            auo: au,
            memory,
            cycle_count: 0,
            cycle_128_count: 0,
            cycle_64_count: 0,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8, muted: bool) {
        self.auo.set_mute(muted);

        let nr52;
        let audio_triggers;

        {
            let mut memory = self.memory.write();

            nr52 = memory.read_byte(0xFF26);
            audio_triggers = memory.audio_has_been_trigered();
        }

        self.cycle_count += last_instruction_cycles as u16;
        self.cycle_128_count += last_instruction_cycles as u16;
        self.cycle_64_count += last_instruction_cycles as u32;

        if self.cycle_count > CYCLES_1_256_SEC {
            self.cycle_count -= CYCLES_1_256_SEC;

            self.auo.step_256();
        }

        if self.cycle_128_count > CYCLES_1_128_SEC {
            self.cycle_128_count -= CYCLES_1_128_SEC;

            self.auo.step_128();
        }

        if self.cycle_64_count > CYCLES_1_64_SEC {
            self.cycle_64_count -= CYCLES_1_64_SEC;

            self.auo.step_64();
        }

        let all_sound_trigger = nr52 & 0b10000000 == 0b10000000;

        if !all_sound_trigger {
            self.stop_all();
            return;
        }

        // Sound 1
        if audio_triggers.0 {
            self.read_pulse(1);
        }

        // Sound 2
        if audio_triggers.1 {
            self.read_pulse(2);
        }

        // Sound 3
        if audio_triggers.2 {
            self.read_wave();
        }

        // TODO: sound 4
    }

    fn stop_all(&mut self) {
        self.auo.stop_all();
    }

    fn read_pulse(&mut self, channel_n: u8) {
        let audio_registers = {
            let memory = self.memory.read();
            memory.read_audio_registers(channel_n)
        };

        let frequency = audio_registers.calculate_frequency();

        let initial_volume_envelope = audio_registers.get_volume_envelope();
        let volume_envelope_direction = audio_registers.get_volume_envelope_direction();

        let volume_envelope_duration_in_1_64_s = audio_registers.get_volume_envelope_duration_64();
        let wave_duty_percent = audio_registers.calculate_wave_duty_percent();
        let sweep = audio_registers.get_sweep();

        let pulse_description = PulseDescription {
            pulse_n: channel_n,
            frequency,
            current_frequency: frequency,
            wave_duty_percent,
            initial_volume_envelope,
            volume_envelope: initial_volume_envelope,
            volume_envelope_direction,
            volume_envelope_duration_in_1_64_s,
            remaining_volume_envelope_duration_in_1_64_s: volume_envelope_duration_in_1_64_s,
            sweep,
        };

        self.auo.play_pulse(&pulse_description);
    }

    fn read_wave(&mut self) {
        let audio_registers;
        let wave;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(3);
            wave = WavePatternRam {
                data: MemorySector::with_data(memory.wave_pattern_ram.data.data.clone()),
            }
        }

        let frequency = audio_registers.calculate_wave_frequency();
        let wave_output_level = audio_registers.get_wave_output_level();

        let wave_description = WaveDescription::new(
            frequency,
            wave_output_level,
            wave,
            audio_registers.get_use_length(),
            audio_registers.get_length(),
            audio_registers.get_should_play(),
        );

        self.auo.play_wave(&wave_description);
    }
}
