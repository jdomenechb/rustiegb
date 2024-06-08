use crate::cartridge::Cartridge;
use crate::io::interrupt_flag::InterruptFlag;
use crate::io::joypad::Joypad;
use crate::io::registers::IORegisters;
use crate::io::stat::STATMode;
use crate::memory::address::Address;
use crate::memory::audio_registers::AudioRegisters;
use crate::memory::bootstrap_rom::BootstrapRom;
use crate::memory::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use crate::memory::internal_ram_memory_sector::InternalRamMemorySector;
use crate::memory::interrupt_enable::InterruptEnable;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::memory::oam_memory_sector::{OamMemorySector, OAM_MEMORY_SECTOR_SIZE};
use crate::memory::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use crate::utils::math::{two_bytes_to_word, word_to_two_bytes};
use crate::{Byte, SignedByte, Word};

pub mod address;
pub mod audio_registers;
pub mod bootstrap_rom;
pub mod internal_ram_8k_memory_sector;
pub mod internal_ram_memory_sector;
pub mod interrupt_enable;
pub mod memory_sector;
pub mod oam_entry;
pub mod oam_memory_sector;
pub mod video_ram_8k_memory_sector;

#[derive(Default, Clone)]
pub struct AudioRegWritten {
    pub control: bool,
    pub length: bool,
    pub sweep_or_wave_onoff: bool,
    pub envelope_or_wave_out_lvl: bool,
    pub frequency_or_poly_counter: bool,
    pub wave_pattern: bool,
}

impl AudioRegWritten {
    pub fn has_change(&self) -> bool {
        self.control
            || self.length
            || self.sweep_or_wave_onoff
            || self.envelope_or_wave_out_lvl
            || self.frequency_or_poly_counter
            || self.wave_pattern
    }
}

#[readonly::make]
#[derive(Default)]
pub struct Memory {
    bootstrap_rom: Option<BootstrapRom>,

    cartridge: Cartridge,

    video_ram: VideoRam8kMemorySector,
    switchable_ram_bank: InternalRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    pub oam_ram: OamMemorySector,

    pub io_registers: IORegisters,

    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    pub interrupt_enable: InterruptEnable,

    // -- Other
    remaining_timer_cycles: u32,
    remaining_div_cycles: u32,
}

impl Memory {
    pub fn new(cartridge: Cartridge, bootstrap_rom: Option<BootstrapRom>) -> Self {
        Self {
            bootstrap_rom,
            cartridge,
            video_ram: VideoRam8kMemorySector::default(),
            switchable_ram_bank: InternalRam8kMemorySector::default(),
            internal_ram_8k: InternalRam8kMemorySector::default(),
            io_registers: IORegisters::default(),
            internal_ram: InternalRamMemorySector::default(),
            interrupt_enable: InterruptEnable::default(),
            oam_ram: OamMemorySector::default(),
            remaining_timer_cycles: 0,
            remaining_div_cycles: 0,
        }
    }

    pub fn read_byte(&self, position: Word) -> Byte {
        let byte = self.internally_read_byte(position).unwrap_or(0xFF);

        match position {
            Address::NR10_SOUND_1_SWEEP => byte | 0b10000000, // 0x80
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => byte | 0b00111111, // 0x3F
            Address::NR13_SOUND_1_FR_LO => 0xFF,
            Address::NR14_SOUND_1_FR_HI => byte | 0b10111111, // 0xBF
            Address::NR20_SOUND_2_UNUSED => 0xFF,
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => byte | 0b00111111, // 0x3F
            Address::NR23_SOUND_2_FR_LO => 0xFF,
            Address::NR24_SOUND_3_FR_HI => byte | 0b10111111, // 0xBF
            0xFF1A => byte | 0b01111111,                      // 0x7F
            0xFF1B => 0xFF,
            0xFF1C => byte | 0b10011111, // 0x9F
            0xFF1D => byte | 0xFF,
            0xFF1E => byte | 0b10111111, // 0xBF
            0xFF1F..=0xFF20 => 0xFF,
            0xFF23 => byte | 0b10111111, // 0xBF
            Address::NR52_SOUND => byte | 0b1110000,
            0xFF27..=0xFF2F => 0xFF,
            _ => byte,
        }
    }

    pub fn internally_read_byte(&self, position: Word) -> Option<Byte> {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return Some(self.bootstrap_rom.as_ref().unwrap().read_byte(position));
        }

        match position {
            0..=0x7FFF => Some(self.cartridge.read_byte(position)),
            0x8000..=0x9FFF => Some(self.video_ram.read_byte(position - 0x8000)),
            0xA000..=0xBFFF => Some(self.cartridge.read_byte(position)),
            0xC000..=0xDFFF => Some(self.internal_ram_8k.read_byte(position - 0xC000)),
            0xE000..=0xFDFF => Some(self.internal_ram_8k.read_byte(position - 0xE000)),
            0xFE00..=0xFE9F => Some(self.oam_ram.read_byte(position - 0xFE00)),
            Address::UNUSED_FF27..=Address::UNUSED_FF2F => None,
            Address::IO_REGISTERS_START..=Address::IO_REGISTERS_END => {
                Some(self.io_registers.read_byte(position))
            }
            0xFF4D => Some(0xFF),
            0xFF80..=0xFFFE => Some(self.internal_ram.read_byte(position - 0xFF80)),
            Address::IE_INTERRUPT_ENABLE => Some((&self.interrupt_enable).into()),
            _ => None,
        }
    }

    pub fn read_signed_byte(&self, position: Word) -> SignedByte {
        self.read_byte(position) as SignedByte
    }

    pub fn read_word(&self, position: Word) -> Word {
        two_bytes_to_word(self.read_byte(position + 1), self.read_byte(position))
    }

    pub fn write_byte(&mut self, position: Word, value: Byte) {
        match position {
            0..=0x7FFF => self.cartridge.write_byte(position, value),
            0x8000..=0x9FFF => self.video_ram.write_byte(position - 0x8000, value),
            0xA000..=0xBFFF => self.cartridge.write_byte(position, value),
            0xC000..=0xDFFF => self.internal_ram_8k.write_byte(position - 0xC000, value),
            0xE000..=0xFDFF => self.internal_ram_8k.write_byte(position - 0xE000, value),
            0xFE00..=0xFE9F => self.oam_ram.write_byte(position - 0xFE00, value),
            0xFEA0..=0xFEFF => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            Address::IO_REGISTERS_START..=Address::IO_REGISTERS_END => {
                self.io_registers.write_byte(position, value)
            }
            0xFF4C..=0xFF7F => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            0xFF80..=0xFFFE => self.internal_ram.write_byte(position - 0xFF80, value),
            Address::IE_INTERRUPT_ENABLE => self.interrupt_enable.update(value),
        };
    }

    pub fn write_word(&mut self, position: Word, value: Word) {
        let bytes = word_to_two_bytes(value);

        self.write_byte(position, bytes.1);
        self.write_byte(position + 1, bytes.0);
    }

    pub fn step(&mut self, last_instruction_cycles: u8) {
        if self.io_registers.dma.step(last_instruction_cycles) {
            let init_address = Word::from(&self.io_registers.dma);

            for i in 0..OAM_MEMORY_SECTOR_SIZE {
                self.oam_ram.write_byte(i, self.read_byte(init_address + i));
            }
        }

        self.remaining_div_cycles += last_instruction_cycles as u32;

        while self.remaining_div_cycles as i16 - 256_i16 > 0 {
            self.io_registers.div = self.io_registers.div.wrapping_add(1);
            self.remaining_div_cycles -= 256_u32;
        }

        if !self.io_registers.timer_control.started {
            self.remaining_timer_cycles = 0;
            return;
        }

        self.remaining_timer_cycles += last_instruction_cycles as u32;

        let divider: u16 = match self.io_registers.timer_control.input_clock_select {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("Invalid input clock select"),
        };

        while self.remaining_timer_cycles as i16 - divider as i16 > 0 {
            let result = self.io_registers.tima.overflowing_add(1);
            self.io_registers.tima = result.0;

            if result.1 {
                self.io_registers.interrupt_flag.set_timer_overflow(true);
                self.io_registers.tima = self.io_registers.tma;
            }

            self.remaining_timer_cycles -= divider as u32;
        }
    }

    pub fn scx(&self) -> Byte {
        self.read_byte(Address::SCX_SCROLL_X)
    }

    pub fn scy(&self) -> Byte {
        self.read_byte(Address::SCY_SCROLL_Y)
    }

    pub fn bgp(&self) -> Byte {
        self.read_byte(Address::BGP_BG_WIN_PALETTE)
    }

    pub fn has_bootstrap_rom(&self) -> bool {
        self.bootstrap_rom.is_some()
    }

    pub fn erase_bootstrap_rom(&mut self) {
        self.bootstrap_rom = None;
    }

    pub fn interrupt_enable(&self) -> &InterruptEnable {
        &self.interrupt_enable
    }

    pub fn interrupt_flag(&mut self) -> &mut InterruptFlag {
        &mut self.io_registers.interrupt_flag
    }

    pub fn joypad(&mut self) -> &mut Joypad {
        &mut self.io_registers.p1
    }

    pub fn oam_ram(&mut self) -> &mut OamMemorySector {
        &mut self.oam_ram
    }

    pub fn set_stat_mode(&mut self, mode: STATMode) {
        match mode {
            STATMode::HBlank => {
                if self.io_registers.stat.mode_0 {
                    self.io_registers.interrupt_flag.set_lcd_stat(true);
                }
            }

            STATMode::VBlank => {
                if self.io_registers.stat.mode_1 {
                    self.io_registers.interrupt_flag.set_lcd_stat(true);
                }

                self.io_registers.interrupt_flag.set_vblank(true);
            }
            STATMode::SearchOamRam => {
                if self.io_registers.stat.mode_2 {
                    self.io_registers.interrupt_flag.set_lcd_stat(true);
                }
            }
            _ => {}
        }

        self.io_registers.stat.set_mode(mode);
    }

    pub fn ly_increment(&mut self) {
        self.io_registers.ly.increment();
        self.determine_ly_interrupt();
    }

    pub fn ly_reset(&mut self) {
        self.io_registers.ly.reset();
        self.determine_ly_interrupt();
    }

    pub fn ly_reset_wo_interrupt(&mut self) {
        self.io_registers.ly.reset();
    }

    fn determine_ly_interrupt(&mut self) {
        let ly = Byte::from(self.io_registers.ly.clone());

        let new_value = ly == self.io_registers.lyc;

        self.io_registers.stat.coincidence_flag = new_value;

        if self.io_registers.stat.lyc_ly_coincidence && new_value {
            self.io_registers.interrupt_flag.set_lcd_stat(true);
        }
    }

    pub fn audio_reg_have_been_written(
        &mut self,
    ) -> (
        AudioRegWritten,
        AudioRegWritten,
        AudioRegWritten,
        AudioRegWritten,
    ) {
        let to_return = (
            self.io_registers.audio_1_reg_written.clone(),
            self.io_registers.audio_2_reg_written.clone(),
            self.io_registers.audio_3_reg_written.clone(),
            self.io_registers.audio_4_reg_written.clone(),
        );

        self.io_registers.audio_1_reg_written = AudioRegWritten::default();
        self.io_registers.audio_2_reg_written = AudioRegWritten::default();
        self.io_registers.audio_3_reg_written = AudioRegWritten::default();
        self.io_registers.audio_4_reg_written = AudioRegWritten::default();

        to_return
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        let mut sweep = None;
        let start_address = match channel {
            1 => {
                sweep = self.internally_read_byte(Address::NR10_SOUND_1_SWEEP);
                Address::NR14_SOUND_1_FR_HI
            }
            2 => Address::NR24_SOUND_3_FR_HI,
            3 => {
                sweep = self.internally_read_byte(0xFF1A);
                0xFF1E
            }
            4 => 0xFF23,
            _ => panic!("Invalid channel provided"),
        };

        AudioRegisters::new(
            self.internally_read_byte(start_address).unwrap(),
            self.internally_read_byte(start_address - 1).unwrap(),
            self.internally_read_byte(start_address - 2).unwrap(),
            self.internally_read_byte(start_address - 3).unwrap(),
            sweep,
        )
    }

    pub fn set_audio_channel_inactive(&mut self, channel_n: Byte) {
        self.io_registers.nr52.set_channel_inactive(channel_n);
    }

    pub fn update_audio_1_frequency(&mut self, frequency: Word) {
        self.io_registers.nr13 = (frequency & 0xFF) as Byte;
        self.io_registers.nr14 =
            (self.io_registers.nr14 & 0b11111000) | ((frequency >> 8) & 0b111) as Byte;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_basic_audio_registers_are_reset(memory: &mut Memory) {
        let items = vec![
            // NR10
            (Address::NR10_SOUND_1_SWEEP, 0x80),
            (Address::NR11_SOUND_1_WAVE_PATTERN_DUTY, 0x3F),
            (Address::NR12_SOUND_1_ENVELOPE, 0x00),
            (Address::NR13_SOUND_1_FR_LO, 0xFF),
            (Address::NR14_SOUND_1_FR_HI, 0xBF),
            // NR20
            (Address::NR20_SOUND_2_UNUSED, 0xFF),
            (Address::NR21_SOUND_2_WAVE_PATTERN_DUTY, 0x3F),
            (Address::NR22_SOUND_2_ENVELOPE, 0x00),
            (Address::NR23_SOUND_2_FR_LO, 0xFF),
            (Address::NR24_SOUND_3_FR_HI, 0xBF),
            // NR30
            (0xFF1A, 0x7F),
            (0xFF1B, 0xFF),
            (0xFF1C, 0x9F),
            (0xFF1D, 0xFF),
            (0xFF1E, 0xBF),
            // NR40
            (0xFF1F, 0xFF),
            (0xFF20, 0xFF),
            (0xFF21, 0x00),
            (0xFF22, 0x00),
            (0xFF23, 0xBF),
            // NR50
            (0xFF24, 0x00),
            (0xFF25, 0x00),
            // NR52 Skipped as it is special
        ];

        for item in items {
            assert_eq!(
                memory.internally_read_byte(item.0),
                Some(0),
                "Wrong internal data when writing register {:X}",
                item.0
            );

            assert_eq!(
                memory.read_byte(item.0),
                item.1,
                "Wrong data when writing register {:X}",
                item.0
            );
        }
    }

    #[test]
    fn test_correct_data_when_writing_audio_registers() {
        let mut memory = Memory::default();

        for position in Address::NR10_SOUND_1_SWEEP..=0xFF25 {
            memory.write_byte(position, 0xFF);
            memory.write_byte(position, 0);
        }

        check_basic_audio_registers_are_reset(&mut memory);

        // NR52
        let position = Address::NR52_SOUND;

        memory.write_byte(position, 0xFF);
        memory.write_byte(position, 0);

        assert_eq!(
            memory.internally_read_byte(position),
            Some(0),
            "Wrong internal data when writing register {:X}",
            position
        );

        assert_eq!(
            memory.read_byte(position),
            0x70,
            "Wrong data when writing register {:X}",
            position
        );

        memory.write_byte(position, 0xFF);

        assert_eq!(
            memory.internally_read_byte(position),
            Some(0b10000000),
            "Wrong internal data when writing register {:X}",
            position
        );

        assert_eq!(
            memory.read_byte(position),
            0xF0,
            "Wrong data when writing register {:X}",
            position
        );

        // Unused registers
        for position in 0xFF27..=0xFF2F {
            memory.write_byte(position, 0xFF);
            memory.write_byte(position, 0);

            assert_eq!(
                memory.internally_read_byte(position),
                None,
                "Wrong internal data when writing register {:X}",
                position
            );

            assert_eq!(
                memory.read_byte(position),
                0xFF,
                "Wrong data when writing register {:X}",
                position
            );
        }

        // WAVE
        for position in 0xFF30..0xFF40 {
            memory.write_byte(position, 0xFF);
            memory.write_byte(position, 0);

            assert_eq!(
                memory.internally_read_byte(position),
                Some(0),
                "Wrong internal data when writing register {:X}",
                position
            );

            assert_eq!(
                memory.read_byte(position),
                0,
                "Wrong data when writing register {:X}",
                position
            );
        }
    }

    #[test]
    fn test_when_sound_is_turned_off_all_audio_registers_are_reset() {
        let mut memory = Memory::default();

        for position in Address::NR10_SOUND_1_SWEEP..=0xFF25 {
            memory.write_byte(position, 0xFF);
        }

        memory.write_byte(Address::NR52_SOUND, 0);
        memory.write_byte(Address::NR52_SOUND, 0b10000000);

        check_basic_audio_registers_are_reset(&mut memory);
    }

    #[test]
    fn test_when_sound_is_turned_off_audio_registers_ignore_writes() {
        let mut memory = Memory::default();

        for position in Address::NR10_SOUND_1_SWEEP..=0xFF25 {
            memory.write_byte(position, 0x00);
        }

        memory.write_byte(Address::NR52_SOUND, 0);

        for position in Address::NR10_SOUND_1_SWEEP..=0xFF25 {
            memory.write_byte(position, 0xFF);
        }

        check_basic_audio_registers_are_reset(&mut memory);
    }

    #[test]
    fn test_unmapped_addresses() {
        let mut addresses = vec![Address::UNUSED_FF03];
        let mut first_range: Vec<Word> = (0xFF08..=0xFF0E).collect();
        addresses.append(&mut first_range);

        let mut memory = Memory::default();

        for address in addresses {
            memory.write_byte(address, 0);

            assert_eq!(memory.read_byte(address), 0xFF);
            assert_eq!(memory.internally_read_byte(address), Some(0xFF));
        }
    }
}
