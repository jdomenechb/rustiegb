use std::sync::Arc;

use parking_lot::RwLock;

use crate::Byte;
use audio_unit_output::AudioUnitOutput;
use description::{PulseDescription, WaveDescription};

use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::memory::Memory;

pub mod audio_unit_output;
mod description;
pub mod sweep;

const CYCLES_1_512_SEC: u16 = 8192;

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

impl From<WaveOutputLevel> for f32 {
    fn from(wol: WaveOutputLevel) -> Self {
        match wol {
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
    frame_step: Byte,
}

impl AudioUnit {
    pub fn new(au: Box<dyn AudioUnitOutput>, memory: Arc<RwLock<Memory>>) -> Self {
        Self {
            auo: au,
            memory,
            cycle_count: 0,
            frame_step: 0,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8, muted: bool) {
        self.auo.set_mute(muted);

        let nr52;
        let audio_triggers;

        {
            let mut memory = self.memory.write();

            nr52 = memory.read_byte(Memory::ADDR_NR52);
            audio_triggers = memory.audio_reg_have_been_written();
        }

        self.clock_frame_sequencer(last_instruction_cycles);

        // NR52 controls the general output
        if nr52 & 0b10000000 != 0b10000000 {
            self.stop_all();
            return;
        }

        // Sound 1
        if audio_triggers.0 .0 || audio_triggers.0 .1 {
            self.update_pulse(1, audio_triggers.0 .1);
        }

        // Sound 2
        if audio_triggers.1 .0 || audio_triggers.1 .1 {
            self.update_pulse(2, audio_triggers.1 .1);
        }

        // Sound 3
        if audio_triggers.2 .0 || audio_triggers.2 .1 {
            self.update_wave(audio_triggers.2 .1);
        }

        // TODO: sound 4

        self.auo.update(self.memory.clone());
    }

    fn clock_frame_sequencer(&mut self, last_instruction_cycles: u8) {
        self.cycle_count += last_instruction_cycles as u16;

        if self.cycle_count > CYCLES_1_512_SEC {
            self.cycle_count -= CYCLES_1_512_SEC;
            self.frame_step = (self.frame_step + 1) % 8;

            if self.frame_step % 2 == 0 {
                self.auo.step_256();
            }

            if self.frame_step == 7 {
                self.auo.step_64();
            }

            if self.frame_step == 2 || self.frame_step == 6 {
                self.auo.step_128()
            }
        }
    }

    fn stop_all(&mut self) {
        self.auo.stop_all();
    }

    fn update_pulse(&mut self, channel_n: u8, only_length: bool) {
        let audio_registers = {
            let memory = self.memory.read();
            memory.read_audio_registers(channel_n)
        };

        if !audio_registers.is_set() {
            self.auo.stop(channel_n);
            return;
        }

        let pulse_length = audio_registers.get_pulse_length();

        if only_length {
            self.auo.reload_length(channel_n, pulse_length);
            return;
        }

        let frequency = audio_registers.get_frequency();

        let initial_volume_envelope = audio_registers.get_volume_envelope();
        let volume_envelope_direction = audio_registers.get_volume_envelope_direction();

        let volume_envelope_duration_in_1_64_s = audio_registers.get_volume_envelope_duration_64();
        let wave_duty_percent = audio_registers.calculate_wave_duty_percent();
        let sweep = audio_registers.get_sweep();

        let pulse_description = PulseDescription::new(
            channel_n,
            frequency,
            wave_duty_percent,
            initial_volume_envelope,
            volume_envelope_direction,
            volume_envelope_duration_in_1_64_s,
            sweep,
            audio_registers.is_length_used(),
            pulse_length,
        );

        self.auo.play_pulse(&pulse_description);
    }

    fn update_wave(&mut self, only_length: bool) {
        let audio_registers;
        let wave;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(3);
            wave = WavePatternRam {
                data: MemorySector::with_data(memory.wave_pattern_ram.data.data.clone()),
            }
        }

        if !audio_registers.is_set() {
            self.auo.stop(3);
            return;
        }

        let length = audio_registers.get_wave_length();

        if only_length {
            self.auo.reload_length(3, length);
            return;
        }

        let frequency = audio_registers.get_frequency();
        let wave_output_level = audio_registers.get_wave_output_level();

        let wave_description = WaveDescription::new(
            frequency,
            wave_output_level,
            wave,
            audio_registers.is_length_used(),
            length,
            audio_registers.get_wave_should_play(),
        );

        self.auo.play_wave(&wave_description);
    }
}
