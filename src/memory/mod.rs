use std::fs::File;
use std::io::Read;

use crate::cartridge::Cartridge;
use crate::math::{two_bytes_to_word, word_to_two_bytes};
use crate::memory::audio_registers::AudioRegisters;
use crate::memory::dma::Dma;
use crate::memory::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use crate::memory::internal_ram_memory_sector::InternalRamMemorySector;
use crate::memory::interrupt_enable::InterruptEnable;
use crate::memory::interrupt_flag::InterruptFlag;
use crate::memory::joypad::Joypad;
use crate::memory::lcdc::Lcdc;
use crate::memory::ly::LY;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::memory::nr52::NR52;
use crate::memory::oam_memory_sector::{OamMemorySector, OAM_MEMORY_SECTOR_SIZE};
use crate::memory::read_only_memory_sector::ReadOnlyMemorySector;
use crate::memory::sio_control::SioControl;
use crate::memory::stat::{STATMode, Stat};
use crate::memory::timer_control::TimerControl;
use crate::memory::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, SignedByte, Word};

pub mod audio_registers;
mod dma;
pub mod internal_ram_8k_memory_sector;
pub mod internal_ram_memory_sector;
mod interrupt_enable;
pub mod interrupt_flag;
pub mod joypad;
pub mod lcdc;
mod ly;
pub mod memory_sector;
pub mod nr52;
pub mod oam_entry;
pub mod oam_memory_sector;
pub mod read_only_memory_sector;
mod sio_control;
pub mod stat;
pub mod timer_control;
pub mod video_ram_8k_memory_sector;
pub mod wave_pattern_ram;

#[readonly::make]
#[derive(Default)]
pub struct Memory {
    bootstrap_rom: Option<ReadOnlyMemorySector>,

    cartridge: Cartridge,

    video_ram: VideoRam8kMemorySector,
    switchable_ram_bank: InternalRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    pub oam_ram: OamMemorySector,
    // FF00
    p1: Joypad,
    // FF01
    serial_transfer_data: Byte,
    // FF02
    sio_control: SioControl,
    // FF04
    div: Byte,
    // FF05
    tima: Byte,
    // FF06
    tma: Byte,
    // FF07
    timer_control: TimerControl,
    // FF0F
    pub interrupt_flag: InterruptFlag,
    // FF10
    nr10: Byte,
    // FF11
    nr11: Byte,
    // FF12
    nr12: Byte,
    // FF13
    nr13: Byte,
    // FF14
    nr14: Byte,
    // FF15
    nr20: Byte,
    // FF16
    nr21: Byte,
    // FF17
    nr22: Byte,
    // FF18
    nr23: Byte,
    // FF19
    nr24: Byte,
    // FF1A
    nr30: Byte,
    // FF1B
    nr31: Byte,
    // FF1C
    nr32: Byte,
    // FF1D
    nr33: Byte,
    // FF1E
    nr34: Byte,
    // FF1F
    nr40: Byte,
    // FF20
    nr41: Byte,
    // FF21
    nr42: Byte,
    // FF22
    nr43: Byte,
    // FF23
    nr44: Byte,
    // FF24
    nr50: Byte,
    // FF25
    nr51: Byte,
    // FF26
    pub nr52: NR52,
    // Wave pattern ram (FF30 - FF3F)
    pub wave_pattern_ram: WavePatternRam,
    // FF40
    pub lcdc: Lcdc,
    // FF41
    pub stat: Stat,
    // FF42 - FF43
    scy: Byte,
    scx: Byte,
    // FF44
    pub ly: LY,
    // FF45
    lyc: Byte,
    // FF46
    dma: Dma,
    // FF47 - FF49
    bgp: Byte,
    obp1: Byte,
    obp2: Byte,
    // FF4A - FF4B
    pub wy: Byte,
    pub wx: Byte,

    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    // FFFF
    pub interrupt_enable: InterruptEnable,

    // -- Other
    remaining_timer_cycles: u32,
    remaining_div_cycles: u32,

    // Audio
    audio_1_control_reg_written: bool,
    audio_2_control_reg_written: bool,
    audio_3_control_reg_written: bool,
    audio_4_control_reg_written: bool,

    audio_1_length_reg_written: bool,
    audio_2_length_reg_written: bool,
    audio_3_length_reg_written: bool,
    audio_4_length_reg_written: bool,
}

impl Memory {
    pub const ADDR_SIO_CONTROL: Word = 0xFF02;
    pub const ADDR_IF: Word = 0xFF0F;
    pub const ADDR_NR10: Word = 0xFF10;
    pub const ADDR_NR52: Word = 0xFF26;
    pub const ADDR_STAT: Word = 0xFF41;
    pub const ADDR_DMA: Word = 0xFF46;
    pub const ADDR_IE: Word = 0xFFFF;

    pub fn new(cartridge: Cartridge, bootstrap: bool) -> Memory {
        let bootstrap_rom = if bootstrap {
            let bootstrap_rom_path = "./DMG_ROM.bin";
            let mut bootstrap_data: Vec<Byte> = Vec::with_capacity(0x256);

            let mut bootstrap_rom_file = File::open(bootstrap_rom_path).expect("file not found");
            bootstrap_rom_file
                .read_to_end(&mut bootstrap_data)
                .expect("Error on reading ROM contents");

            Some(ReadOnlyMemorySector::new(&bootstrap_data))
        } else {
            None
        };

        Memory {
            bootstrap_rom,
            cartridge,
            video_ram: VideoRam8kMemorySector::default(),
            switchable_ram_bank: InternalRam8kMemorySector::default(),
            internal_ram_8k: InternalRam8kMemorySector::default(),
            p1: Joypad::new(),
            serial_transfer_data: 0,
            sio_control: SioControl::default(),
            div: 0,
            tima: 0,
            tma: 0,
            timer_control: TimerControl::default(),
            interrupt_flag: InterruptFlag::new(),
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0x00,
            nr14: 0xBF,
            nr20: 0xFF,
            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0x00,
            nr24: 0xBF,
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9f,
            nr33: 0x00,
            nr34: 0xBF,
            nr40: 0xFF,
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
            nr50: 0x77,
            nr51: 0xf3,
            nr52: NR52::default(),
            wave_pattern_ram: WavePatternRam::default(),
            lcdc: Lcdc::new(),
            stat: Stat::default(),
            scy: 0x00,
            scx: 0x00,
            ly: LY::default(),
            lyc: 0x00,
            dma: Dma::default(),
            bgp: 0xFC,
            obp1: 0xFF,
            obp2: 0xFF,
            wy: 0x00,
            wx: 0x00,
            internal_ram: InternalRamMemorySector::default(),
            interrupt_enable: InterruptEnable::default(),
            oam_ram: OamMemorySector::default(),
            remaining_timer_cycles: 0,
            remaining_div_cycles: 0,
            audio_1_control_reg_written: false,
            audio_2_control_reg_written: false,
            audio_3_control_reg_written: false,
            audio_4_control_reg_written: false,
            audio_1_length_reg_written: false,
            audio_2_length_reg_written: false,
            audio_3_length_reg_written: false,
            audio_4_length_reg_written: false,
        }
    }

    pub fn read_byte(&self, position: Word) -> Byte {
        let byte = self.internally_read_byte(position).unwrap_or(0xFF);

        match position {
            Self::ADDR_NR10 => byte | 0b10000000, // 0x80
            0xFF11 => byte | 0b00111111,          // 0x3F
            0xFF13 => 0xFF,
            0xFF14 => byte | 0b10111111, // 0xBF
            0xFF15 => 0xFF,
            0xFF16 => byte | 0b00111111, // 0x3F
            0xFF18 => 0xFF,
            0xFF19 => byte | 0b10111111, // 0xBF
            0xFF1A => byte | 0b01111111, // 0x7F
            0xFF1B => 0xFF,
            0xFF1C => byte | 0b10011111, // 0x9F
            0xFF1D => byte | 0xFF,
            0xFF1E => byte | 0b10111111, // 0xBF
            0xFF1F..=0xFF20 => 0xFF,
            0xFF23 => byte | 0b10111111, // 0xBF
            Self::ADDR_NR52 => byte | 0b1110000,
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
            0xA000..=0xBFFF => Some(self.switchable_ram_bank.read_byte(position - 0xA000)),
            0xC000..=0xDFFF => Some(self.internal_ram_8k.read_byte(position - 0xC000)),
            0xE000..=0xFDFF => Some(self.internal_ram_8k.read_byte(position - 0xE000)),
            0xFE00..=0xFE9F => Some(self.oam_ram.read_byte(position - 0xFE00)),
            0xFF00 => Some(self.p1.to_byte()),
            0xFF01 => Some(self.serial_transfer_data),
            Self::ADDR_SIO_CONTROL => Some((&self.sio_control).into()),
            0xFF03 => Some(0xFF),
            0xFF04 => Some(self.div),
            0xFF05 => Some(self.tima),
            0xFF06 => Some(self.tma),
            0xFF08..=0xFF0E => Some(0xFF),
            Self::ADDR_IF => Some((&self.interrupt_flag).into()),
            Self::ADDR_NR10 => Some(self.nr10),
            0xFF11 => Some(self.nr11),
            0xFF12 => Some(self.nr12),
            0xFF13 => Some(self.nr13),
            0xFF14 => Some(self.nr14),
            0xFF15 => Some(self.nr20),
            0xFF16 => Some(self.nr21),
            0xFF17 => Some(self.nr22),
            0xFF18 => Some(self.nr23),
            0xFF19 => Some(self.nr24),
            0xFF1A => Some(self.nr30),
            0xFF1B => Some(self.nr31),
            0xFF1C => Some(self.nr32),
            0xFF1D => Some(self.nr33),
            0xFF1E => Some(self.nr34),
            0xFF1F => Some(self.nr40),
            0xFF20 => Some(self.nr41),
            0xFF21 => Some(self.nr42),
            0xFF22 => Some(self.nr43),
            0xFF23 => Some(self.nr44),
            0xFF24 => Some(self.nr50),
            0xFF25 => Some(self.nr51),
            Self::ADDR_NR52 => Some((&self.nr52).into()),
            0xFF30..=0xFF3F => Some(self.wave_pattern_ram.read_byte(position - 0xFF30)),
            0xFF40 => Some((&self.lcdc).into()),
            Self::ADDR_STAT => Some((&self.stat).into()),
            0xFF42 => Some(self.scy),
            0xFF43 => Some(self.scx),
            0xFF44 => Some(self.ly.clone().into()),
            0xFF45 => Some(self.lyc),
            Self::ADDR_DMA => Some((&self.dma).into()),
            0xFF47 => Some(self.bgp),
            0xFF48 => Some(self.obp1),
            0xFF49 => Some(self.obp2),
            0xFF4A => Some(self.wy),
            0xFF4B => Some(self.wx),
            0xFF4D => Some(0xFF),
            0xFF80..=0xFFFE => Some(self.internal_ram.read_byte(position - 0xFF80)),
            Self::ADDR_IE => Some((&self.interrupt_enable).into()),
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
            0xA000..=0xBFFF => self
                .switchable_ram_bank
                .write_byte(position - 0xA000, value),
            0xC000..=0xDFFF => self.internal_ram_8k.write_byte(position - 0xC000, value),
            0xE000..=0xFDFF => self.internal_ram_8k.write_byte(position - 0xE000, value),
            0xFE00..=0xFE9F => self.oam_ram.write_byte(position - 0xFE00, value),
            0xFEA0..=0xFEFF => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            0xFF00 => self.p1.from_byte(value),
            0xFF01 => self.serial_transfer_data = value,
            Self::ADDR_SIO_CONTROL => self.sio_control = value.into(),
            0xFF03 => println!("Attempt to write at an unused RAM position {:X}", position),
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.timer_control = value.into(),
            0xFF08..=0xFF0E => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            Self::ADDR_IF => self.interrupt_flag = value.into(),
            Self::ADDR_NR10 => {
                if self.nr52.is_on() {
                    self.nr10 = value;
                }
            }
            0xFF11 => {
                if self.nr52.is_on() {
                    self.nr11 = value;
                    self.audio_1_length_reg_written = true;
                }
            }
            0xFF12 => {
                if self.nr52.is_on() {
                    self.nr12 = value;
                }
            }
            0xFF13 => {
                if self.nr52.is_on() {
                    self.nr13 = value;
                }
            }
            0xFF14 => {
                if !self.nr52.is_on() {
                    return;
                }

                if value & 0b10000000 == 0b10000000 {
                    self.audio_1_control_reg_written = true;
                    self.nr52.set_channel_active(1);
                }

                self.nr14 = value;
            }
            0xFF15 => {
                if self.nr52.is_on() {
                    self.nr20 = value;
                }
            }
            0xFF16 => {
                if self.nr52.is_on() {
                    self.nr21 = value;
                    self.audio_2_length_reg_written = true;
                }
            }
            0xFF17 => {
                if self.nr52.is_on() {
                    self.nr22 = value;
                }
            }
            0xFF18 => {
                if self.nr52.is_on() {
                    self.nr23 = value;
                }
            }
            0xFF19 => {
                if !self.nr52.is_on() {
                    return;
                }

                if value & 0b10000000 == 0b10000000 {
                    self.audio_2_control_reg_written = true;
                    self.nr52.set_channel_active(2);
                }

                self.nr24 = value;
            }
            0xFF1A => {
                if self.nr52.is_on() {
                    self.nr30 = value;
                }
            }
            0xFF1B => {
                if self.nr52.is_on() {
                    self.nr31 = value;
                    self.audio_3_length_reg_written = true;
                }
            }
            0xFF1C => {
                if self.nr52.is_on() {
                    self.nr32 = value;
                }
            }
            0xFF1D => {
                if self.nr52.is_on() {
                    self.nr33 = value;
                }
            }
            0xFF1E => {
                if !self.nr52.is_on() {
                    return;
                }

                if value & 0b10000000 == 0b10000000 {
                    self.audio_3_control_reg_written = true;
                    self.nr52.set_channel_active(3);
                }

                self.nr34 = value;
            }
            0xFF1F => {
                if self.nr52.is_on() {
                    self.nr40 = value;
                }
            }
            0xFF20 => {
                if self.nr52.is_on() {
                    self.nr41 = value;
                    self.audio_4_length_reg_written = true;
                }
            }
            0xFF21 => {
                if self.nr52.is_on() {
                    self.nr42 = value;
                }
            }
            0xFF22 => {
                if self.nr52.is_on() {
                    self.nr43 = value;
                }
            }
            0xFF23 => {
                if !self.nr52.is_on() {
                    return;
                }

                if value & 0b10000000 == 0b10000000 {
                    self.audio_4_control_reg_written = true;
                    self.nr52.set_channel_active(4);
                }

                self.nr44 = value;
            }
            0xFF24 => {
                if self.nr52.is_on() {
                    self.nr50 = value;
                }
            }
            0xFF25 => {
                if self.nr52.is_on() {
                    self.nr51 = value;
                }
            }
            Self::ADDR_NR52 => {
                self.nr52 = (value & 0b10000000).into();

                if self.nr52.is_on() {
                    self.nr10 = 0;
                    self.nr11 = 0;
                    self.nr12 = 0;
                    self.nr13 = 0;
                    self.nr14 = 0;
                    self.nr20 = 0;
                    self.nr21 = 0;
                    self.nr22 = 0;
                    self.nr23 = 0;
                    self.nr24 = 0;
                    self.nr30 = 0;
                    self.nr31 = 0;
                    self.nr32 = 0;
                    self.nr33 = 0;
                    self.nr34 = 0;
                    self.nr40 = 0;
                    self.nr41 = 0;
                    self.nr42 = 0;
                    self.nr43 = 0;
                    self.nr44 = 0;
                    self.nr50 = 0;
                    self.nr51 = 0;
                }
            }
            0xFF27..=0xFF2F => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            0xFF30..=0xFF3F => self.wave_pattern_ram.write_byte(position - 0xFF30, value),
            0xFF40 => self.lcdc = value.into(),
            Self::ADDR_STAT => self.stat = value.into(),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => self.ly = value.into(),
            0xFF45 => self.lyc = value,
            Self::ADDR_DMA => self.dma = value.into(),
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp1 = value,
            0xFF49 => self.obp2 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            0xFF4C..=0xFF7F => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            0xFF80..=0xFFFE => self.internal_ram.write_byte(position - 0xFF80, value),
            Self::ADDR_IE => self.interrupt_enable = value.into(),
        };
    }

    pub fn write_word(&mut self, position: Word, value: Word) {
        let bytes = word_to_two_bytes(value);

        self.write_byte(position, bytes.1);
        self.write_byte(position + 1, bytes.0);
    }

    pub fn step(&mut self, last_instruction_cycles: u8) {
        if self.dma.step(last_instruction_cycles) {
            let init_address = Word::from(&self.dma);

            for i in 0..OAM_MEMORY_SECTOR_SIZE {
                self.oam_ram.write_byte(i, self.read_byte(init_address + i));
            }
        }

        self.remaining_div_cycles += last_instruction_cycles as u32;

        while self.remaining_div_cycles as i16 - 256_i16 > 0 {
            self.div = self.div.wrapping_add(1);
            self.remaining_div_cycles -= 256_u32;
        }

        if !self.timer_control.started {
            self.remaining_timer_cycles = 0;
            return;
        }

        self.remaining_timer_cycles += last_instruction_cycles as u32;

        let divider: u16 = match self.timer_control.input_clock_select {
            0 => 1024,
            1 => 16,
            2 => 64,
            3 => 256,
            _ => panic!("Invalid input clock select"),
        };

        while self.remaining_timer_cycles as i16 - divider as i16 > 0 {
            let result = self.tima.overflowing_add(1);
            self.tima = result.0;

            if result.1 {
                self.interrupt_flag.set_timer_overflow(true);
                self.tima = self.tma;
            }

            self.remaining_timer_cycles -= divider as u32;
        }
    }

    pub fn scx(&self) -> Byte {
        self.read_byte(0xFF43)
    }

    pub fn scy(&self) -> Byte {
        self.read_byte(0xFF42)
    }

    pub fn bgp(&self) -> Byte {
        self.read_byte(0xFF47)
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
        &mut self.interrupt_flag
    }

    pub fn joypad(&mut self) -> &mut Joypad {
        &mut self.p1
    }

    pub fn oam_ram(&mut self) -> &mut OamMemorySector {
        &mut self.oam_ram
    }

    pub fn set_stat_mode(&mut self, mode: STATMode) {
        match mode {
            STATMode::HBlank => {
                if self.stat.mode_0 {
                    self.interrupt_flag.set_lcd_stat(true);
                }
            }

            STATMode::VBlank => {
                if self.stat.mode_1 {
                    self.interrupt_flag.set_lcd_stat(true);
                }

                self.interrupt_flag().set_vblank(true);
            }
            STATMode::SearchOamRam => {
                if self.stat.mode_2 {
                    self.interrupt_flag.set_lcd_stat(true);
                }
            }
            _ => {}
        }

        self.stat.set_mode(mode);
    }

    pub fn ly_increment(&mut self) {
        self.ly.increment();
        self.determine_ly_interrupt();
    }

    pub fn ly_reset(&mut self) {
        self.ly.reset();
        self.determine_ly_interrupt();
    }

    fn determine_ly_interrupt(&mut self) {
        let ly = Byte::from(self.ly.clone());

        let new_value = ly == self.lyc;

        self.stat.coincidence_flag = new_value;

        if self.stat.lyc_ly_coincidence && new_value {
            self.interrupt_flag.set_lcd_stat(true);
        }
    }

    pub fn audio_reg_have_been_written(
        &mut self,
    ) -> ((bool, bool), (bool, bool), (bool, bool), (bool, bool)) {
        let to_return = (
            (
                self.audio_1_control_reg_written,
                self.audio_1_length_reg_written,
            ),
            (
                self.audio_2_control_reg_written,
                self.audio_2_length_reg_written,
            ),
            (
                self.audio_3_control_reg_written,
                self.audio_3_length_reg_written,
            ),
            (
                self.audio_4_control_reg_written,
                self.audio_4_length_reg_written,
            ),
        );

        self.audio_1_control_reg_written = false;
        self.audio_2_control_reg_written = false;
        self.audio_3_control_reg_written = false;
        self.audio_4_control_reg_written = false;
        self.audio_1_length_reg_written = false;
        self.audio_2_length_reg_written = false;
        self.audio_3_length_reg_written = false;
        self.audio_4_length_reg_written = false;

        to_return
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        let mut sweep = None;
        let start_address = match channel {
            1 => {
                sweep = self.internally_read_byte(Self::ADDR_NR10);
                0xFF14
            }
            2 => 0xFF19,
            3 => {
                sweep = self.internally_read_byte(0xFF1A);
                0xFF1E
            }
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
        self.nr52.set_channel_inactive(channel_n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_basic_audio_registers_are_reset(memory: &mut Memory) {
        let items = vec![
            // NR10
            (Memory::ADDR_NR10, 0x80),
            (0xFF11, 0x3F),
            (0xFF12, 0x00),
            (0xFF13, 0xFF),
            (0xFF14, 0xBF),
            // NR20
            (0xFF15, 0xFF),
            (0xFF16, 0x3F),
            (0xFF17, 0x00),
            (0xFF18, 0xFF),
            (0xFF19, 0xBF),
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

        for position in Memory::ADDR_NR10..=0xFF25 {
            memory.write_byte(position, 0xFF);
            memory.write_byte(position, 0);
        }

        check_basic_audio_registers_are_reset(&mut memory);

        // NR52
        let position = Memory::ADDR_NR52;

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

        for position in Memory::ADDR_NR10..=0xFF25 {
            memory.write_byte(position, 0xFF);
        }

        memory.write_byte(Memory::ADDR_NR52, 0);
        memory.write_byte(Memory::ADDR_NR52, 0b10000000);

        check_basic_audio_registers_are_reset(&mut memory);
    }

    #[test]
    fn test_when_sound_is_turned_off_audio_registers_ignore_writes() {
        let mut memory = Memory::default();

        memory.write_byte(Memory::ADDR_NR52, 0);

        for position in Memory::ADDR_NR10..=0xFF25 {
            memory.write_byte(position, 0xFF);
        }

        check_basic_audio_registers_are_reset(&mut memory);
    }

    #[test]
    fn test_unmapped_addresses() {
        let mut addresses = vec![0xFF03];
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
