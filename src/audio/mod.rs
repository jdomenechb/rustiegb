pub mod audio_unit_output;

use crate::memory::memory::Memory;
use crate::{Byte, Word};
use audio_unit_output::AudioUnitOutput;
use std::cell::RefCell;
use std::rc::Rc;

const CYCLES_1_256_SEC: u16 = 16384;
const CYCLES_1_64_SEC: u32 = 16384 * 4;

#[derive(Eq, PartialEq, Copy, Clone)]
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

pub struct PulseDescription {
    pub pulse_n: u8,
    pub frequency: f32,
    pub wave_duty_percent: f32,
    pub initial_volume_envelope: Byte,
    pub volume_envelope: Byte,
    pub volume_envelope_direction: VolumeEnvelopeDirection,
    pub volume_envelope_duration_in_1_64_s: u8,
    pub remaining_volume_envelope_duration_in_1_64_s: u8,
}

impl PulseDescription {
    fn step_64(&mut self) {
        if self.volume_envelope_duration_in_1_64_s == 0 {
            return;
        }

        if self.remaining_volume_envelope_duration_in_1_64_s == 0 {
            match self.volume_envelope_direction {
                VolumeEnvelopeDirection::UP => {
                    if self.volume_envelope < 0xF {
                        self.volume_envelope += 1;
                    }
                }
                VolumeEnvelopeDirection::DOWN => {
                    if self.volume_envelope > 0 {
                        self.volume_envelope -= 1;
                    }
                }
            }

            self.remaining_volume_envelope_duration_in_1_64_s =
                self.volume_envelope_duration_in_1_64_s;

            return;
        }

        self.remaining_volume_envelope_duration_in_1_64_s -= 1;

        return;
    }

    fn exchange(&mut self, other: &Self) {
        self.pulse_n = other.pulse_n;
        self.frequency = other.frequency;
        self.wave_duty_percent = other.wave_duty_percent;
        self.initial_volume_envelope = other.initial_volume_envelope;
        self.volume_envelope = other.volume_envelope;
        self.volume_envelope_direction = other.volume_envelope_direction;
        self.volume_envelope_duration_in_1_64_s = other.volume_envelope_duration_in_1_64_s;
        self.remaining_volume_envelope_duration_in_1_64_s =
            other.remaining_volume_envelope_duration_in_1_64_s;
    }
}

impl PartialEq for PulseDescription {
    fn eq(&self, other: &Self) -> bool {
        return other.pulse_n == self.pulse_n
            && other.frequency == self.frequency
            && other.wave_duty_percent == self.wave_duty_percent
            && other.initial_volume_envelope == self.initial_volume_envelope
            && other.volume_envelope_direction == self.volume_envelope_direction
            && other.volume_envelope_duration_in_1_64_s == self.volume_envelope_duration_in_1_64_s
            && other.remaining_volume_envelope_duration_in_1_64_s
                == self.remaining_volume_envelope_duration_in_1_64_s;
    }
}

impl Default for PulseDescription {
    fn default() -> Self {
        Self {
            pulse_n: 0,
            frequency: 0.0,
            wave_duty_percent: 0.0,
            initial_volume_envelope: 0,
            volume_envelope: 0,
            volume_envelope_direction: VolumeEnvelopeDirection::UP,
            volume_envelope_duration_in_1_64_s: 0,
            remaining_volume_envelope_duration_in_1_64_s: 0,
        }
    }
}

pub struct AudioUnit {
    auo: Box<dyn AudioUnitOutput>,
    memory: Rc<RefCell<Memory>>,

    cycle_count: u16,
    cycle_64_count: u32,
}

impl AudioUnit {
    pub fn new(au: Box<dyn AudioUnitOutput>, memory: Rc<RefCell<Memory>>) -> Self {
        Self {
            auo: au,
            memory,
            cycle_count: 0,
            cycle_64_count: 0,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8) {
        let nr52;
        let audio_triggers;

        {
            let mut memory = self.memory.borrow_mut();

            nr52 = memory.read_byte(0xFF26);
            audio_triggers = memory.audio_has_been_trigered();
        }

        self.cycle_count += last_instruction_cycles as u16;
        self.cycle_64_count += last_instruction_cycles as u32;

        if self.cycle_count > CYCLES_1_256_SEC {
            self.cycle_count -= CYCLES_1_256_SEC;

            // TODO
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

    fn stop_all(&mut self) {
        self.auo.stop_all();
    }

    fn read_pulse(
        &mut self,
        pulse_n: u8,
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
            let memory = self.memory.borrow();

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

        let initial_volume_envelope = (volume_reg >> 4) & 0xF;
        let volume_envelope_direction =
            VolumeEnvelopeDirection::from(volume_reg & 0b1000 == 0b1000);

        let volume_envelope_duration_in_1_64_s = volume_reg & 0b111;

        let pulse_description = PulseDescription {
            pulse_n,
            frequency,
            wave_duty_percent,
            initial_volume_envelope,
            volume_envelope: initial_volume_envelope,
            volume_envelope_direction,
            volume_envelope_duration_in_1_64_s,
            remaining_volume_envelope_duration_in_1_64_s: volume_envelope_duration_in_1_64_s,
        };

        self.auo.play_pulse(&pulse_description);
    }

    pub fn toggle_mute(&mut self) {
        self.auo.toggle_mute();
    }
}
