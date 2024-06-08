use crate::bus::address::Address;
use crate::cartridge::Cartridge;
use crate::io::registers::IORegisters;
use crate::memory::bootstrap_rom::BootstrapRom;
use crate::memory::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use crate::memory::internal_ram_memory_sector::InternalRamMemorySector;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::memory::oam_memory_sector::{OamMemorySector, OAM_MEMORY_SECTOR_SIZE};
use crate::memory::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use crate::utils::math::{two_bytes_to_word, word_to_two_bytes};
use crate::{Byte, SignedByte, Word};
use parking_lot::RwLock;
use std::sync::Arc;

pub mod bootstrap_rom;
pub mod internal_ram_8k_memory_sector;
pub mod internal_ram_memory_sector;
pub mod memory_sector;
pub mod oam_entry;
pub mod oam_memory_sector;
pub mod video_ram_8k_memory_sector;

#[readonly::make]
#[derive(Default)]
pub struct Memory {
    bootstrap_rom: Option<BootstrapRom>,

    cartridge: Cartridge,

    video_ram: VideoRam8kMemorySector,
    switchable_ram_bank: InternalRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    pub oam_ram: OamMemorySector,

    pub io_registers: Arc<RwLock<IORegisters>>,

    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
}

impl Memory {
    pub fn new(
        io_registers: Arc<RwLock<IORegisters>>,
        cartridge: Cartridge,
        bootstrap_rom: Option<BootstrapRom>,
    ) -> Self {
        Self {
            bootstrap_rom,
            cartridge,
            video_ram: VideoRam8kMemorySector::default(),
            switchable_ram_bank: InternalRam8kMemorySector::default(),
            internal_ram_8k: InternalRam8kMemorySector::default(),
            io_registers,
            internal_ram: InternalRamMemorySector::default(),
            oam_ram: OamMemorySector::default(),
        }
    }

    pub fn read_byte(&self, position: Word) -> Byte {
        let byte = self.internally_read_byte(position).unwrap_or(0xFF);

        match position {
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
                Some(self.io_registers.read().read_byte(position))
            }
            0xFF4D => Some(0xFF),
            0xFF80..=0xFFFE => Some(self.internal_ram.read_byte(position - 0xFF80)),
            Address::IE_INTERRUPT_ENABLE => Some(self.io_registers.read().read_byte(position)),
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
                self.io_registers.write().write_byte(position, value)
            }
            0xFF4C..=0xFF7F => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            0xFF80..=0xFFFE => self.internal_ram.write_byte(position - 0xFF80, value),
            Address::IE_INTERRUPT_ENABLE => self.io_registers.write().write_byte(position, value),
        };
    }

    pub fn write_word(&mut self, position: Word, value: Word) {
        let bytes = word_to_two_bytes(value);

        self.write_byte(position, bytes.1);
        self.write_byte(position + 1, bytes.0);
    }

    pub fn step(&mut self, last_instruction_cycles: u8) {
        let dma_init_address = {
            let mut io_registers = self.io_registers.write();
            io_registers.step(last_instruction_cycles)
        };

        if dma_init_address.is_some() {
            let dma_init_address = dma_init_address.unwrap();

            for i in 0..OAM_MEMORY_SECTOR_SIZE {
                self.oam_ram
                    .write_byte(i, self.read_byte(dma_init_address + i));
            }
        }
    }

    pub fn has_bootstrap_rom(&self) -> bool {
        self.bootstrap_rom.is_some()
    }

    pub fn erase_bootstrap_rom(&mut self) {
        self.bootstrap_rom = None;
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
            /*
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

             */
        ];

        for item in items {
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
            memory.read_byte(position),
            0x70,
            "Wrong data when writing register {:X}",
            position
        );

        memory.write_byte(position, 0xFF);

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
