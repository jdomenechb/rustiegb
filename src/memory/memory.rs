use super::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use super::internal_ram_memory_sector::InternalRamMemorySector;
use super::interrupt_flag::InterruptFlag;
use super::lcdc::LCDC;
use super::read_only_memory_sector::ReadOnlyMemorySector;
use super::stat::STAT;
use super::timer_control::TimerControl;
use super::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use crate::memory::joypad::Joypad;
use crate::memory::memory_sector::ReadMemory;
use crate::memory::memory_sector::WriteMemory;
use crate::memory::oam_memory_sector::OamMemorySector;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, SignedByte, Word};
use std::fs::File;
use std::io::Read;

#[derive(Default)]
pub struct Memory {
    bootstrap_rom: Option<ReadOnlyMemorySector>,
    rom: ReadOnlyMemorySector,
    video_ram: VideoRam8kMemorySector,
    switchable_ram_bank: InternalRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    oam_ram: OamMemorySector,
    // FF00
    p1: Joypad,
    // FF01
    serial_transfer_data: Byte,
    // FF02
    sio_control: Byte,
    // FF04
    div: Byte,
    // FF06
    tma: Byte,
    // FF07
    timer_control: TimerControl,
    // FF0F
    interrupt_flag: InterruptFlag,
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
    nr52: Byte,
    // Wave pattern ram (FF30 - FF3F)
    wave_pattern_ram: WavePatternRam,
    // FF40
    pub lcdc: LCDC,
    // FF41
    pub stat: STAT,
    // FF42 - FF43
    scy: Byte,
    scx: Byte,
    // FF44
    pub ly: Byte,
    // FF46
    dma: Byte,
    // FF47 - FF49
    bgp: Byte,
    obp1: Byte,
    obp2: Byte,
    // FF4A - FF4B
    wy: Byte,
    wx: Byte,
    // FF4D - CGB ONLY
    key1: Byte,

    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    // FFFF
    interrupt_enable: InterruptFlag,
}

impl Memory {
    pub fn new(rom_path: &str, bootstrap: bool) -> Memory {
        let mut data: Vec<Byte> = Vec::with_capacity(0x8000);
        let mut rom_file = File::open(rom_path).expect("file not found");
        rom_file
            .read_to_end(&mut data)
            .expect("Error on reading ROM contents");

        let bootstrap_rom;

        if bootstrap {
            let bootstrap_rom_path = "./DMG_ROM.bin";
            let mut bootstrap_data: Vec<Byte> = Vec::with_capacity(0x256);

            let mut bootstrap_rom_file = File::open(bootstrap_rom_path).expect("file not found");
            bootstrap_rom_file
                .read_to_end(&mut bootstrap_data)
                .expect("Error on reading ROM contents");

            bootstrap_rom = Some(ReadOnlyMemorySector::new(bootstrap_data));
        } else {
            bootstrap_rom = None;
        }

        return Memory {
            bootstrap_rom,
            rom: ReadOnlyMemorySector::new(data),
            video_ram: VideoRam8kMemorySector::default(),
            switchable_ram_bank: InternalRam8kMemorySector::default(),
            internal_ram_8k: InternalRam8kMemorySector::default(),
            p1: Joypad::new(),
            serial_transfer_data: 0,
            sio_control: 0,
            div: 0,
            tma: 0,
            timer_control: TimerControl::new(),
            interrupt_flag: InterruptFlag::new(),
            nr10: 0x80,
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0x00,
            nr14: 0xBF,
            nr21: 0x3F,
            nr22: 0x00,
            nr23: 0x00,
            nr24: 0xBF,
            nr30: 0x7F,
            nr31: 0xFF,
            nr32: 0x9f,
            nr33: 0x00,
            nr34: 0xBF,
            nr41: 0xFF,
            nr42: 0x00,
            nr43: 0x00,
            nr44: 0xBF,
            nr50: 0x77,
            nr51: 0xf3,
            nr52: 0xf1,
            wave_pattern_ram: WavePatternRam::default(),
            lcdc: LCDC::new(),
            stat: STAT::default(),
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            dma: 0x00,
            bgp: 0xFC,
            obp1: 0xFF,
            obp2: 0xFF,
            wy: 0x00,
            wx: 0x00,
            key1: 0x00,
            internal_ram: InternalRamMemorySector::default(),
            interrupt_enable: InterruptFlag::new(),
            oam_ram: OamMemorySector::default(),
        };
    }

    pub fn read_byte(&self, position: Word) -> Byte {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return self.bootstrap_rom.as_ref().unwrap().read_byte(position);
        }

        // ROM
        if position < 0x8000 {
            return self.rom.read_byte(position);
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.read_byte(position - 0x8000);
        }

        // 8k switchable RAM bank
        if position >= 0xA000 && position < 0xC000 {
            return self.switchable_ram_bank.read_byte(position - 0xA000);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.read_byte(position - 0xC000);
        }

        // Echo of Internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.read_byte(position - 0xE000);
        }

        // OAM Ram
        if position >= 0xFE00 && position < 0xFEA0 {
            return self.oam_ram.read_byte(position - 0xFE00);
        }

        // P1
        if position == 0xFF00 {
            return self.p1.to_byte();
        }

        // Serial transfer data
        if position == 0xFF01 {
            return self.serial_transfer_data;
        }

        // SIO control
        if position == 0xFF02 {
            return self.sio_control;
        }

        // DIV register
        if position == 0xFF04 {
            return self.div;
        }

        if position == 0xFF06 {
            return self.tma;
        }

        // NR10
        if position == 0xFF10 {
            return self.nr10;
        }

        // NR11
        if position == 0xFF11 {
            return self.nr11;
        }

        // NR12
        if position == 0xFF12 {
            return self.nr12;
        }

        // NR13 is not readable

        // NR14
        if position == 0xFF14 {
            return self.nr14;
        }

        // NR21
        if position == 0xFF16 {
            return self.nr21;
        }

        // NR22
        if position == 0xFF17 {
            return self.nr22;
        }

        // NR23 is not readable

        // NR24
        if position == 0xFF19 {
            return self.nr24;
        }

        // NR30
        if position == 0xFF1A {
            return self.nr30;
        }

        // NR31
        if position == 0xFF1B {
            return self.nr31;
        }

        // NR32
        if position == 0xFF1C {
            return self.nr32;
        }

        // NR33 is not readable

        // NR34
        if position == 0xFF1E {
            return self.nr34;
        }

        // NR41
        if position == 0xFF20 {
            return self.nr41;
        }

        // NR42
        if position == 0xFF21 {
            return self.nr42;
        }

        // NR43
        if position == 0xFF22 {
            return self.nr43;
        }

        // NR44
        if position == 0xFF23 {
            return self.nr44;
        }

        // NR50
        if position == 0xFF24 {
            return self.nr50;
        }

        // NR51
        if position == 0xFF25 {
            return self.nr51;
        }

        // NR52
        if position == 0xFF26 {
            return self.nr52;
        }

        // Wave pattern RAM
        if position >= 0xFF30 && position < 0xFF40 {
            return self.wave_pattern_ram.read_byte(position - 0xFF30);
        }

        // LCDC
        if position == 0xFF40 {
            return (&self.lcdc).into();
        }

        // STAT
        if position == 0xFF41 {
            return (&self.stat).into();
        }

        // SCY
        if position == 0xFF42 {
            return self.scy;
        }

        // SCX
        if position == 0xFF43 {
            return self.scx;
        }

        // LY
        if position == 0xFF44 {
            return self.ly;
        }

        // DMA
        if position == 0xFF46 {
            return self.dma;
        }

        // BGP
        if position == 0xFF47 {
            return self.bgp;
        }

        // OBP1
        if position == 0xFF48 {
            return self.obp1;
        }

        // OBP2
        if position == 0xFF49 {
            return self.obp2;
        }

        // Window Y
        if position == 0xFF4A {
            return self.wy;
        }

        // Window X
        if position == 0xFF4B {
            return self.wx;
        }

        // KEY 1
        if position == 0xFF4D {
            return self.key1;
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_byte(position - 0xFF80);
        }

        // Interrupt enable
        if position == 0xFFFF {
            return (&self.interrupt_enable).into();
        }

        panic!("ERROR: Memory address {:X} not readable", position);
    }

    pub fn read_signed_byte(&self, position: Word) -> SignedByte {
        self.read_byte(position) as SignedByte
    }

    pub fn read_word(&self, position: Word) -> Word {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return self.bootstrap_rom.as_ref().unwrap().read_word(position);
        }

        // ROM
        if position < 0x8000 {
            return self.rom.read_word(position);
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.read_word(position - 0x8000);
        }

        // 8k switchable RAM bank
        if position >= 0xA000 && position < 0xC000 {
            return self.switchable_ram_bank.read_word(position - 0xA000);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.read_word(position - 0xC000);
        }

        // Echo of internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.read_word(position - 0xE000);
        }

        if position >= 0xFE00 && position < 0xFEA0 {
            return self.oam_ram.read_word(position - 0xFE00);
        }

        // Wave pattern RAM
        if position >= 0xFF30 && position < 0xFF40 {
            return self.wave_pattern_ram.read_word(position - 0xFF30);
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_word(position - 0xFF80);
        }

        panic!("ERROR: Memory address {:X} not readable", position);
    }

    pub fn write_byte(&mut self, position: Word, value: Byte) {
        // ROM
        if position < 0x8000 {
            println!(
                "Attempt to write at Memory {:X}. ROM is not writable!!!",
                position
            );
            return;
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            self.video_ram.write_byte(position - 0x8000, value);
            return;
        }

        // 8k switchable RAM bank
        if position >= 0xA000 && position < 0xC000 {
            self.switchable_ram_bank
                .write_byte(position - 0xA000, value);
            return;
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            self.internal_ram_8k.write_byte(position - 0xC000, value);
            return;
        }

        // Echo of internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            self.internal_ram_8k.write_byte(position - 0xE000, value);
            return;
        }

        if position >= 0xFE00 && position < 0xFEA0 {
            self.oam_ram.write_byte(position - 0xFE00, value);
            return;
        }

        if position >= 0xFEA0 && position < 0xFF00 {
            println!("Attempt to write at an unused RAM position {:X}", position);
            return;
        }

        // P1
        if position == 0xFF00 {
            self.p1.from_byte(value);
            return;
        }

        // Serial transfer data
        if position == 0xFF01 {
            self.serial_transfer_data = value;
            return;
        }

        // Serial transfer data
        if position == 0xFF02 {
            self.sio_control = value;
            return;
        }

        // DIV register
        if position == 0xFF04 {
            self.div = 0;
            return;
        }

        if position == 0xFF06 {
            self.tma = value;
            return;
        }

        // Timer Control
        if position == 0xFF07 {
            self.timer_control.from_byte(value);
            return;
        }

        // Interrupt Flag
        if position == 0xFF0F {
            self.interrupt_flag = value.into();
            return;
        }

        // NR10
        if position == 0xFF10 {
            self.nr10 = value;
            return;
        }

        // NR11
        if position == 0xFF11 {
            self.nr11 = value;
            return;
        }

        // NR12
        if position == 0xFF12 {
            self.nr12 = value;
            return;
        }

        // NR13
        if position == 0xFF13 {
            self.nr13 = value;
            return;
        }

        // NR14
        if position == 0xFF14 {
            self.nr14 = value;
            return;
        }

        // NR21
        if position == 0xFF16 {
            self.nr21 = value;
            return;
        }

        // NR22
        if position == 0xFF17 {
            self.nr22 = value;
            return;
        }

        // NR23
        if position == 0xFF18 {
            self.nr23 = value;
            return;
        }

        // NR24
        if position == 0xFF19 {
            self.nr24 = value;
            return;
        }

        // NR30
        if position == 0xFF1A {
            self.nr30 = value;
            return;
        }

        // NR31
        if position == 0xFF1B {
            self.nr31 = value;
            return;
        }

        // NR32
        if position == 0xFF1C {
            self.nr32 = value;
            return;
        }

        // NR33
        if position == 0xFF1D {
            self.nr33 = value;
            return;
        }

        // NR34
        if position == 0xFF1E {
            self.nr34 = value;
            return;
        }

        // NR41
        if position == 0xFF20 {
            self.nr41 = value;
            return;
        }

        // NR42
        if position == 0xFF21 {
            self.nr42 = value;
            return;
        }

        // NR43
        if position == 0xFF22 {
            self.nr43 = value;
            return;
        }

        // NR44
        if position == 0xFF23 {
            self.nr44 = value;
            return;
        }

        // NR50
        if position == 0xFF24 {
            self.nr50 = value;
            return;
        }

        // NR51
        if position == 0xFF25 {
            self.nr51 = value;
            return;
        }

        // NR52
        if position == 0xFF26 {
            self.nr52 = value;
            return;
        }

        if position >= 0xFF30 && position < 0xFF40 {
            self.wave_pattern_ram.write_byte(position - 0xFF30, value);
            return;
        }

        // LCDC
        if position == 0xFF40 {
            self.lcdc = value.into();
            return;
        }

        // STAT
        if position == 0xFF41 {
            self.stat = value.into();
            return;
        }

        // SCY
        if position == 0xFF42 {
            self.scy = value;
            return;
        }

        // SCX
        if position == 0xFF43 {
            self.scx = value;
            return;
        }

        // LY
        if position == 0xFF44 {
            self.ly = value;
            return;
        }

        // DMA
        if position == 0xFF46 {
            self.dma = value;

            // DMA Transfer
            let init_address = (self.dma as Word) << 8 & 0xFF00;

            for i in (0..0x8C).step_by(2) {
                self.oam_ram.write_word(i, self.read_word(init_address + i));
            }

            self.dma = 0;

            return;
        }

        // BGP
        if position == 0xFF47 {
            self.bgp = value;
            return;
        }

        // OBP1
        if position == 0xFF48 {
            self.obp1 = value;
            return;
        }

        // OBP2
        if position == 0xFF49 {
            self.obp2 = value;
            return;
        }

        // Window Y
        if position == 0xFF4A {
            self.wy = value;
            return;
        }

        // Window X
        if position == 0xFF4B {
            self.wx = value;
            return;
        }

        // Empty but unusable for I/O
        if position == 0xFF4C {
            println!("Attempt to write at an unused RAM position {:X}", position);
            return;
        }

        // KEY 1
        if position == 0xFF4D {
            self.key1 = value;
            return;
        }

        // Empty but unusable for I/O
        if position >= 0xFF4E && position < 0xFF80 {
            println!("Attempt to write at an unused RAM position {:X}", position);
            return;
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            self.internal_ram.write_byte(position - 0xFF80, value);
            return;
        }

        // Interrupt enable
        if position == 0xFFFF {
            self.interrupt_enable = value.into();
            return;
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }

    pub fn write_word(&mut self, position: Word, value: Word) {
        // ROM
        if position < 0x8000 {
            println!(
                "Attempt to write at Memory {:X}. ROM is not writable!!!",
                position
            );
            return;
        }

        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.write_word(position - 0x8000, value);
        }

        // Internal RAM 8k
        if position >= 0xA000 && position < 0xC000 {
            return self
                .switchable_ram_bank
                .write_word(position - 0xA000, value);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.write_word(position - 0xC000, value);
        }

        // Echo of internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.write_word(position - 0xE000, value);
        }

        if position >= 0xFE00 && position < 0xFEA0 {
            return self.oam_ram.write_word(position - 0xE000, value);
        }

        if position >= 0xFEA0 && position < 0xFF00 {
            println!("Attempt to write at an unused RAM position {:X}", position);
            return;
        }

        if position >= 0xFF30 && position < 0xFF40 {
            return self.wave_pattern_ram.write_word(position - 0xFF30, value);
        }

        if position >= 0xFF4C && position < 0xFF80 {
            println!("Attempt to write at an unused RAM position {:X}", position);
            return;
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.write_word(position - 0xFF80, value);
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }

    // TODO: Implement DIV register
    // pub fn step(&mut self, last_instruction_cycles: i16) {
    // }

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

    pub fn interrupt_enable(&self) -> &InterruptFlag {
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
}
