use crate::bus::address::Address;
use crate::cartridge::Cartridge;
use crate::io::registers::IORegisters;
use crate::memory::bootstrap_rom::BootstrapRom;
use crate::memory::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use crate::memory::internal_ram_memory_sector::InternalRamMemorySector;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::memory::oam_memory_sector::{OAM_MEMORY_SECTOR_SIZE, OamMemorySector};
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
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < Address::CARTRIDGE_START {
            return self.bootstrap_rom.as_ref().unwrap().read_byte(position);
        }

        match position {
            0..=0x7FFF => self.cartridge.read_byte(position),
            0x8000..=0x9FFF => self.video_ram.read_byte(position - 0x8000),
            0xA000..=0xBFFF => self.cartridge.read_byte(position),
            0xC000..=0xDFFF => self.internal_ram_8k.read_byte(position - 0xC000),
            0xE000..=0xFDFF => self.internal_ram_8k.read_byte(position - 0xE000),
            0xFE00..=0xFE9F => self.oam_ram.read_byte(position - 0xFE00),
            Address::IO_REGISTERS_START..=Address::IO_REGISTERS_END => {
                self.io_registers.read().read_byte(position)
            }
            0xFF80..=0xFFFE => self.internal_ram.read_byte(position - 0xFF80),
            Address::IE_INTERRUPT_ENABLE => self.io_registers.read().read_byte(position),
            _ => 0xFF,
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
            Address::IO_REGISTERS_START..=Address::IO_REGISTERS_END => {
                self.io_registers.write().write_byte(position, value)
            }
            0xFF80..=0xFFFE => self.internal_ram.write_byte(position - 0xFF80, value),
            Address::IE_INTERRUPT_ENABLE => self.io_registers.write().write_byte(position, value),
            _ => {
                println!("Attempt to write at an unused RAM position {position:X}")
            }
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

        if let Some(dma_init_address) = dma_init_address {
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

    #[test]
    fn test_unmapped_addresses() {
        let mut addresses = vec![Address::UNUSED_FF03];
        let mut first_range: Vec<Word> = (0xFF08..=0xFF0E).collect();
        addresses.append(&mut first_range);

        let mut memory = Memory::default();

        for address in addresses {
            memory.write_byte(address, 0);

            assert_eq!(memory.read_byte(address), 0xFF);
        }
    }
}
