pub mod audio_unit_output;

use crate::memory::memory::Memory;
use crate::memory::memory_sector::MemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::Byte;
use audio_unit_output::AudioUnitOutput;
use parking_lot::RwLock;
use std::sync::Arc;

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

#[readonly::make]
pub struct WaveDescription {
    pub frequency: f32,
    pub output_level: WaveOutputLevel,
    pub wave: WavePatternRam,
}

impl WaveDescription {
    fn exchange(&mut self, other: &Self) {
        self.frequency = other.frequency;
        self.output_level = other.output_level.clone();
        self.wave = WavePatternRam {
            data: MemorySector::with_data(other.wave.data.data.clone()),
        };
    }
}

impl PartialEq for WaveDescription {
    fn eq(&self, other: &Self) -> bool {
        return other.frequency == self.frequency && other.output_level == self.output_level;
    }
}

impl Default for WaveDescription {
    fn default() -> Self {
        Self {
            frequency: 0.0,
            output_level: WaveOutputLevel::Mute,
            wave: WavePatternRam::default(),
        }
    }
}

pub struct AudioUnit {
    auo: Box<dyn AudioUnitOutput>,
    memory: Arc<RwLock<Memory>>,

    cycle_count: u16,
    cycle_64_count: u32,
}

impl AudioUnit {
    pub fn new(au: Box<dyn AudioUnitOutput>, memory: Arc<RwLock<Memory>>) -> Self {
        Self {
            auo: au,
            memory,
            cycle_count: 0,
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

        // TODO: sound 4

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
    }

    fn stop_all(&mut self) {
        self.auo.stop_all();
    }

    fn read_pulse(&mut self, channel_n: u8) {
        let audio_registers;

        {
            let memory = self.memory.read();
            audio_registers = memory.read_audio_registers(channel_n);
        }

        let frequency = audio_registers.calculate_frequency();

        let initial_volume_envelope = audio_registers.get_volume_envelope();
        let volume_envelope_direction = audio_registers.get_volume_envelope_direction();

        let volume_envelope_duration_in_1_64_s = audio_registers.get_volume_envelope_duration_64();
        let wave_duty_percent = audio_registers.calculate_wave_duty_percent();

        let pulse_description = PulseDescription {
            pulse_n: channel_n,
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

        let wave_description = WaveDescription {
            frequency,
            output_level: wave_output_level,
            wave,
        };

        self.auo.play_wave(&wave_description);
    }
}
