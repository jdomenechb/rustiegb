use super::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use super::internal_ram_memory_sector::InternalRamMemorySector;
use super::interrupt_flag::InterruptFlag;
use super::lcdc::LCDC;
use super::read_only_memory_sector::ReadOnlyMemorySector;
use super::stat::STAT;
use super::timer_control::TimerControl;
use super::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use crate::cartridge::Cartridge;
use crate::memory::audio_registers::AudioRegisters;
use crate::memory::joypad::Joypad;
use crate::memory::ly::LY;
use crate::memory::memory_sector::ReadMemory;
use crate::memory::memory_sector::WriteMemory;
use crate::memory::oam_memory_sector::OamMemorySector;
use crate::memory::stat::STATMode;
use crate::memory::wave_pattern_ram::WavePatternRam;
use crate::{Byte, SignedByte, Word};
use std::fs::File;
use std::io::Read;

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
    sio_control: Byte,
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
    pub wave_pattern_ram: WavePatternRam,
    // FF40
    pub lcdc: LCDC,
    // FF41
    pub stat: STAT,
    // FF42 - FF43
    scy: Byte,
    scx: Byte,
    // FF44
    pub ly: LY,
    // FF45
    lyc: Byte,
    // FF46
    dma: Byte,
    // FF47 - FF49
    bgp: Byte,
    obp1: Byte,
    obp2: Byte,
    // FF4A - FF4B
    pub wy: Byte,
    pub wx: Byte,
    // FF4D - CGB ONLY
    key1: Byte,

    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    // FFFF
    pub interrupt_enable: InterruptFlag,

    // -- Other
    remaining_timer_cycles: u32,
    remaining_div_cycles: u32,

    // Audio
    audio_1_triggered: bool,
    audio_2_triggered: bool,
    audio_3_triggered: bool,
    audio_4_triggered: bool,
}

impl Memory {
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

        return Memory {
            bootstrap_rom,
            cartridge,
            video_ram: VideoRam8kMemorySector::default(),
            switchable_ram_bank: InternalRam8kMemorySector::default(),
            internal_ram_8k: InternalRam8kMemorySector::default(),
            p1: Joypad::new(),
            serial_transfer_data: 0,
            sio_control: 0,
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
            ly: LY::default(),
            lyc: 0x00,
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
            remaining_timer_cycles: 0,
            remaining_div_cycles: 0,
            audio_1_triggered: false,
            audio_2_triggered: false,
            audio_3_triggered: false,
            audio_4_triggered: false,
        };
    }

    pub fn read_byte(&self, position: Word) -> Byte {
        let byte = match position {
            0xFF13 => None,
            0xFF18 => None,
            0xFF1D => None,
            _ => self.internally_read_byte(position),
        };

        if byte.is_none() {
            panic!("ERROR: Memory address {:X} not readable", position);
        }

        byte.unwrap()
    }

    pub fn internally_read_byte(&self, position: Word) -> Option<Byte> {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return Some(self.bootstrap_rom.as_ref().unwrap().read_byte(position));
        }

        // ROM
        if position < 0x8000 {
            return Some(self.cartridge.read_byte(position));
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            return Some(self.video_ram.read_byte(position - 0x8000));
        }

        // 8k switchable RAM bank
        if position >= 0xA000 && position < 0xC000 {
            return Some(self.switchable_ram_bank.read_byte(position - 0xA000));
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return Some(self.internal_ram_8k.read_byte(position - 0xC000));
        }

        // Echo of Internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            return Some(self.internal_ram_8k.read_byte(position - 0xE000));
        }

        // OAM Ram
        if position >= 0xFE00 && position < 0xFEA0 {
            return Some(self.oam_ram.read_byte(position - 0xFE00));
        }

        // P1
        if position == 0xFF00 {
            return Some(self.p1.to_byte());
        }

        // Serial transfer data
        if position == 0xFF01 {
            return Some(self.serial_transfer_data);
        }

        // SIO control
        if position == 0xFF02 {
            return Some(self.sio_control);
        }

        // DIV register
        if position == 0xFF04 {
            return Some(self.div);
        }

        // TIMA
        if position == 0xFF05 {
            return Some(self.tima);
        }

        // TMA
        if position == 0xFF06 {
            return Some(self.tma);
        }

        // Interrupt flag
        if position == 0xFF0F {
            return Some((&self.interrupt_flag).into());
        }

        // NR10
        if position == 0xFF10 {
            return Some(self.nr10);
        }

        // NR11
        if position == 0xFF11 {
            return Some(self.nr11);
        }

        // NR12
        if position == 0xFF12 {
            return Some(self.nr12);
        }

        // NR13
        if position == 0xFF13 {
            return Some(self.nr13);
        }

        // NR14
        if position == 0xFF14 {
            return Some(self.nr14);
        }

        // NR21
        if position == 0xFF16 {
            return Some(self.nr21);
        }

        // NR22
        if position == 0xFF17 {
            return Some(self.nr22);
        }

        // NR22
        if position == 0xFF18 {
            return Some(self.nr23);
        }

        // NR24
        if position == 0xFF19 {
            return Some(self.nr24);
        }

        // NR30
        if position == 0xFF1A {
            return Some(self.nr30);
        }

        // NR31
        if position == 0xFF1B {
            return Some(self.nr31);
        }

        // NR32
        if position == 0xFF1C {
            return Some(self.nr32);
        }

        // NR33
        if position == 0xFF1D {
            return Some(self.nr33);
        }

        // NR34
        if position == 0xFF1E {
            return Some(self.nr34);
        }

        // NR41
        if position == 0xFF20 {
            return Some(self.nr41);
        }

        // NR42
        if position == 0xFF21 {
            return Some(self.nr42);
        }

        // NR43
        if position == 0xFF22 {
            return Some(self.nr43);
        }

        // NR44
        if position == 0xFF23 {
            return Some(self.nr44);
        }

        // NR50
        if position == 0xFF24 {
            return Some(self.nr50);
        }

        // NR51
        if position == 0xFF25 {
            return Some(self.nr51);
        }

        // NR52
        if position == 0xFF26 {
            return Some(self.nr52);
        }

        // Wave pattern RAM
        if position >= 0xFF30 && position < 0xFF40 {
            return Some(self.wave_pattern_ram.read_byte(position - 0xFF30));
        }

        // LCDC
        if position == 0xFF40 {
            return Some((&self.lcdc).into());
        }

        // STAT
        if position == 0xFF41 {
            return Some((&self.stat).into());
        }

        // SCY
        if position == 0xFF42 {
            return Some(self.scy);
        }

        // SCX
        if position == 0xFF43 {
            return Some(self.scx);
        }

        // LY
        if position == 0xFF44 {
            return Some(self.ly.clone().into());
        }

        // LYC
        if position == 0xFF45 {
            return Some(self.lyc);
        }

        // DMA
        if position == 0xFF46 {
            return Some(self.dma);
        }

        // BGP
        if position == 0xFF47 {
            return Some(self.bgp);
        }

        // OBP1
        if position == 0xFF48 {
            return Some(self.obp1);
        }

        // OBP2
        if position == 0xFF49 {
            return Some(self.obp2);
        }

        // Window Y
        if position == 0xFF4A {
            return Some(self.wy);
        }

        // Window X
        if position == 0xFF4B {
            return Some(self.wx);
        }

        // KEY 1
        if position == 0xFF4D {
            return Some(self.key1);
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return Some(self.internal_ram.read_byte(position - 0xFF80));
        }

        // Interrupt enable
        if position == 0xFFFF {
            return Some((&self.interrupt_enable).into());
        }

        None
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
            return self.cartridge.read_word(position);
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
            self.cartridge.write_byte(position, value);
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

        // TIMA
        if position == 0xFF05 {
            self.tima = value;
            return;
        }

        // TMA
        if position == 0xFF06 {
            self.tma = value;
            return;
        }

        // Timer Control
        if position == 0xFF07 {
            self.timer_control = value.into();
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
            if value & 0b10000000 == 0b10000000 {
                self.audio_1_triggered = true;
            }

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
            if value & 0b10000000 == 0b10000000 {
                self.audio_2_triggered = true;
            }

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
            if value & 0b10000000 == 0b10000000 {
                self.audio_3_triggered = true;
            }

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
            if value & 0b10000000 == 0b10000000 {
                self.audio_4_triggered = true;
            }

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
            self.ly = value.into();
            return;
        }

        // LYC
        if position == 0xFF45 {
            self.lyc = value;
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
            self.cartridge.write_word(position, value);
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

    pub fn step(&mut self, last_instruction_cycles: u8) {
        self.remaining_div_cycles += last_instruction_cycles as u32;

        while self.remaining_div_cycles as i16 - 256 as i16 > 0 {
            self.div = self.div.wrapping_add(1);
            self.remaining_div_cycles -= 256 as u32;
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

        if self.stat.lyc_ly_coincidence {
            self.interrupt_flag.set_lcd_stat(
                (ly == self.lyc && self.stat.coincidence_flag)
                    || (ly != self.lyc && !self.stat.coincidence_flag),
            );
        }
    }

    pub fn audio_has_been_trigered(&mut self) -> (bool, bool, bool, bool) {
        let to_return = (
            self.audio_1_triggered,
            self.audio_2_triggered,
            self.audio_3_triggered,
            self.audio_4_triggered,
        );

        self.audio_1_triggered = false;
        self.audio_2_triggered = false;
        self.audio_3_triggered = false;
        self.audio_4_triggered = false;

        to_return
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        let mut sweep = None;
        let start_address = match channel {
            1 => {
                sweep = self.internally_read_byte(0xFF10);
                0xFF14
            }
            2 => 0xFF19,
            3 => 0xFF1E,
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
}
