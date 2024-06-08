use std::sync::Arc;

use parking_lot::RwLock;

use crate::io::wave_pattern_ram::WavePatternRam;
use crate::{Byte, CpalAudioUnitOutput};

use crate::memory::address::Address;
use crate::memory::memory_sector::MemorySector;
use crate::memory::{AudioRegWritten, Memory};

pub mod audio_unit_output;
mod noise;
pub mod pulse;
mod registers;
pub mod volume_envelope;
pub mod wave;

const CYCLES_1_512_SEC: u16 = 8192;

pub struct AudioUnit {
    auo: CpalAudioUnitOutput,
    memory: Arc<RwLock<Memory>>,

    cycle_count: u16,
    frame_step: Byte,
    was_stopped: bool,
}

impl AudioUnit {
    pub fn new(au: CpalAudioUnitOutput, memory: Arc<RwLock<Memory>>) -> Self {
        Self {
            auo: au,
            memory,
            cycle_count: 0,
            frame_step: 7,
            was_stopped: true,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8, muted: bool) {
        self.auo.set_mute(muted);

        let nr52;
        let audio_triggers;

        {
            let mut memory = self.memory.write();

            nr52 = memory.read_byte(Address::NR52_SOUND);
            audio_triggers = memory.audio_reg_have_been_written();
        }

        // NR52 controls the general output
        if nr52 & 0b10000000 != 0b10000000 {
            self.auo.stop_all();
            self.was_stopped = true;
            return;
        }

        if self.was_stopped {
            self.was_stopped = false;
            self.frame_step = 7;
        }

        self.clock_frame_sequencer(last_instruction_cycles);

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

        if changes.length {
            self.auo.update_length(channel_n, audio_registers.length);
            return;
        }

        if changes.sweep_or_wave_onoff {
            self.auo.update_sweep(audio_registers.sweep.unwrap());
            return;
        }

        if changes.envelope_or_wave_out_lvl {
            self.auo
                .update_envelope(channel_n, audio_registers.envelope_or_wave_out_lvl);
            return;
        }

        if changes.frequency_or_poly_counter {
            self.auo
                .update_frequency(channel_n, audio_registers.frequency_or_poly_counter);
            return;
        }

        if changes.control {
            self.auo.update_control(
                channel_n,
                audio_registers.control,
                self.next_frame_step_is_length(),
            );
        }
    }

    fn update_wave(&mut self, changes: &AudioRegWritten) {
        let audio_registers;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(3);
        }

        if changes.sweep_or_wave_onoff {
            self.auo.update_wave_onoff(audio_registers.sweep.unwrap());
            return;
        }

        if changes.length {
            self.auo.update_length(3, audio_registers.length);
            return;
        }

        if changes.envelope_or_wave_out_lvl {
            self.auo
                .update_wave_output_level(audio_registers.envelope_or_wave_out_lvl);
            return;
        }

        if changes.frequency_or_poly_counter {
            self.auo
                .update_frequency(3, audio_registers.frequency_or_poly_counter);
            return;
        }

        if changes.control {
            self.auo
                .update_control(3, audio_registers.control, self.next_frame_step_is_length());
            return;
        }

        if changes.wave_pattern {
            self.auo.update_wave_pattern(WavePatternRam {
                data: MemorySector::with_data(
                    self.memory
                        .read()
                        .io_registers
                        .read()
                        .wave_pattern_ram
                        .data
                        .data
                        .clone(),
                ),
            });
        }
    }

    fn update_noise(&mut self, changes: &AudioRegWritten) {
        let audio_registers = {
            let memory = self.memory.read();
            memory.read_audio_registers(4)
        };

        if changes.length {
            self.auo.update_length(4, audio_registers.length);
            return;
        }

        if changes.envelope_or_wave_out_lvl {
            self.auo
                .update_envelope(4, audio_registers.envelope_or_wave_out_lvl);
            return;
        }

        if changes.frequency_or_poly_counter {
            self.auo
                .update_noise_poly_counter(audio_registers.frequency_or_poly_counter);
            return;
        }

        if changes.control {
            self.auo
                .update_control(4, audio_registers.control, self.next_frame_step_is_length());
        }
    }

    fn next_frame_step_is_length(&self) -> bool {
        self.frame_step % 2 == 1
    }
}
