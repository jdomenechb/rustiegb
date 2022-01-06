use std::sync::Arc;

use parking_lot::RwLock;

use crate::{Byte, CpalAudioUnitOutput};
use noise::NoiseDescription;
use pulse::PulseDescription;
use volume_envelope::VolumeEnvelopeDescription;
use wave::WaveDescription;

use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::memory::{AudioRegWritten, Memory};

pub mod audio_unit_output;
mod noise;
pub mod pulse;
pub mod volume_envelope;
pub mod wave;

const CYCLES_1_512_SEC: u16 = 8192;

pub struct AudioUnit {
    auo: CpalAudioUnitOutput,
    memory: Arc<RwLock<Memory>>,

    cycle_count: u16,
    frame_step: Byte,
}

impl AudioUnit {
    pub fn new(au: CpalAudioUnitOutput, memory: Arc<RwLock<Memory>>) -> Self {
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
            self.auo.stop_all();
            return;
        }

        // Sound 1
        if audio_triggers.0.has_change() {
            self.update_pulse(1, &audio_triggers.0);
        }

        // Sound 2
        if audio_triggers.1.has_change() {
            self.update_pulse(2, &audio_triggers.1);
        }

        // Sound 3
        if audio_triggers.2.has_change() {
            self.update_wave(&audio_triggers.2);
        }

        // Sound 4
        if audio_triggers.3.has_change() {
            self.update_noise(&audio_triggers.3);
        }

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
                self.auo.step_128(self.memory.clone())
            }
        }
    }

    fn update_pulse(&mut self, channel_n: u8, changes: &AudioRegWritten) {
        let audio_registers = {
            let memory = self.memory.read();
            memory.read_audio_registers(channel_n)
        };

        let pulse_length = audio_registers.get_pulse_or_noise_length();

        if changes.length {
            self.auo.reload_length(channel_n, pulse_length);
            return;
        }

        let sweep = audio_registers.get_sweep();

        if changes.sweep {
            self.auo.reload_sweep(sweep);
            return;
        }

        let pulse_description = PulseDescription::new(
            audio_registers.is_set(),
            audio_registers.get_frequency(),
            audio_registers.calculate_wave_duty(),
            VolumeEnvelopeDescription::new(
                audio_registers.get_volume_envelope(),
                audio_registers.get_volume_envelope_direction(),
                audio_registers.get_volume_envelope_duration_64(),
            ),
            sweep,
            audio_registers.is_length_used(),
            pulse_length,
        );

        self.auo.play_pulse(channel_n, &pulse_description);
    }

    fn update_wave(&mut self, changes: &AudioRegWritten) {
        let audio_registers;
        let wave;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(3);
            wave = WavePatternRam {
                data: MemorySector::with_data(memory.wave_pattern_ram.data.data.clone()),
            }
        }

        let length = audio_registers.get_wave_length();

        if changes.length {
            self.auo.reload_length(3, length);
            return;
        }

        let frequency = audio_registers.get_frequency();
        let wave_output_level = audio_registers.get_wave_output_level();

        let wave_description = WaveDescription::new(
            audio_registers.is_set(),
            frequency,
            wave_output_level,
            wave,
            audio_registers.is_length_used(),
            length,
            audio_registers.get_wave_should_play(),
        );

        self.auo.play_wave(&wave_description);
    }

    fn update_noise(&mut self, changes: &AudioRegWritten) {
        let audio_registers;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(4);
        }

        let length = audio_registers.get_pulse_or_noise_length();

        if changes.length {
            self.auo.reload_length(4, length);
            return;
        }

        let noise_description = NoiseDescription::new(
            audio_registers.is_set(),
            VolumeEnvelopeDescription::new(
                audio_registers.get_volume_envelope(),
                audio_registers.get_volume_envelope_direction(),
                audio_registers.get_volume_envelope_duration_64(),
            ),
            audio_registers.get_poly_shift_clock_freq(),
            audio_registers.get_poly_step(),
            audio_registers.get_poly_div_ratio(),
            audio_registers.is_length_used(),
            audio_registers.get_pulse_or_noise_length(),
        );

        self.auo.play_noise(&noise_description);
    }
}
