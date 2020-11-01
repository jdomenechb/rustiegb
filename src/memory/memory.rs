use super::read_only_memory_sector::ReadOnlyMemorySector;
use super::internal_ram_memory_sector::InternalRamMemorySector;
use super::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use super::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use super::timer_control::TimerControl;
use super::interrupt_flag::InterruptFlag;
use super::lcdc::LCDC;
use super::stat::STAT;

use std::fs::File;
use std::io::Read;

pub struct Memory {
    bootstrap_rom: Option<ReadOnlyMemorySector>,
    rom: ReadOnlyMemorySector,
    video_ram: VideoRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    
    // FF01
    serial_transfer_data: u8,
    // FF02
    sio_control: u8,
    // FF07
    timer_control: TimerControl,
    // FF0F
    interrupt_flag: InterruptFlag,
    // FF11
    nr11: u8,
    // FF12
    nr12: u8,
    // FF13
    nr13: u8,
    // FF14
    nr14: u8,
    // FF24
    nr50: u8,
    // FF25
    nr51: u8,
    // FF26
    nr52: u8,
    // FF40
    pub lcdc: LCDC,
    // FF41
    pub stat: STAT,
    // FF42 - FF43
    scy: u8, 
    scx: u8,
    // FF44
    pub ly: u8,
    // FF47 - FF49
    bgp: u8,
    obp1: u8,
    obp2: u8,
    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    // FFFF
    interrupt_enable: InterruptFlag,
}

impl Memory {
    pub fn new(rom_path: &str, bootstrap: bool) -> Memory {
        let mut data:Vec<u8> = Vec::with_capacity(0x8000);
        let mut rom_file = File::open(rom_path).expect("file not found");
        rom_file.read_to_end(&mut data).expect("Error on reading ROM contents");

        let bootstrap_rom;

        if bootstrap {
            let bootstrap_rom_path = "./DMG_ROM.bin";
            let mut bootstrap_data: Vec<u8> = Vec::with_capacity(0x256);

            let mut bootstrap_rom_file = File::open(bootstrap_rom_path).expect("file not found");
            bootstrap_rom_file.read_to_end(&mut bootstrap_data).expect("Error on reading ROM contents");

            bootstrap_rom = Some(ReadOnlyMemorySector::new(bootstrap_data));
        } else {
            bootstrap_rom = None;
        }

        return Memory {
            bootstrap_rom,
            rom: ReadOnlyMemorySector::new(data),
            video_ram: VideoRam8kMemorySector::new(),
            internal_ram_8k: InternalRam8kMemorySector::new(),
            serial_transfer_data: 0,
            sio_control: 0,
            timer_control: TimerControl::new(),
            interrupt_flag: InterruptFlag::new(),
            nr11: 0xBF,
            nr12: 0xF3,
            nr13: 0x00,
            nr14: 0xBF,
            nr50: 0x77,
            nr51: 0xf3,
            nr52: 0xf1,
            lcdc: LCDC::new(),
            stat: STAT::new(),
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            bgp: 0xFC,
            obp1: 0xFF,
            obp2: 0xFF,
            internal_ram: InternalRamMemorySector::new(),
            interrupt_enable: InterruptFlag::new()
        };
    }

    pub fn read_8(&self, position: u16) -> u8 {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return self.bootstrap_rom.as_ref().unwrap().read_8(position);
        }

        // ROM
        if position < 0x8000 {
            return self.rom.read_8(position);
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.read_8(position - 0x8000);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.read_8(position - 0xC000);
        }

        // Echo of Internal RAM
        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.read_8(position - 0xE000);
        }

        // Serial transfer data
        if position == 0xFF01 {
            return self.serial_transfer_data;
        }

        // SIO control
        if position == 0xFF02 {
            return self.sio_control;
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

        // STAT
        if position == 0xFF41 {
            return self.stat.to_u8();
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

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_8(position - 0xFF80);
        }

        panic!("ERROR: Memory address {:X} not readable", position);
    }

    pub fn read_8_signed(&self, position: u16) -> i8 {
        let value :u8 = self.read_8(position);

        return value as i8;
    }

    pub fn read_16(&self, position: u16) -> u16 {
        // Bootstrap rom
        if self.bootstrap_rom.is_some() && position < 0x100 {
            return self.bootstrap_rom.as_ref().unwrap().read_16(position);
        }

        // ROM
        if position < 0x8000 {
            return self.rom.read_16(position);
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.read_16(position - 0x8000);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.read_16(position - 0xC000);
        }

        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.read_16(position - 0xE000);
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_16(position - 0xFF80);
        }

        panic!("ERROR: Memory address {:X} not readable", position);
    }

    pub fn write_8(&mut self, position: u16, value: u8) {
        // ROM
        if position < 0x8000 {
            println!("Attempt to write at Memory {:X}. ROM is not writable!!!", position);
            return;
        }

        // Video RAM
        if position >= 0x8000 && position < 0xA000 {
            self.video_ram.write_8(position - 0x8000, value);
            return;
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            self.internal_ram_8k.write_8(position - 0xC000, value);
            return;
        }

        if position >= 0xE000 && position < 0xFE00 {
            self.internal_ram_8k.write_8(position - 0xE000, value);
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

        // Timer Control
        if position == 0xFF07 {
            self.timer_control.from_u8(value);
            return;
        }

        // Interrupt Flag
        if position == 0xFF0F {
            self.interrupt_flag.from_u8(value);
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

        // LCDC
        if position == 0xFF40 {
            self.lcdc.from_u8(value);
            return;
        }

        // STAT
        if position == 0xFF41 {
            self.stat.from_u8(value);
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

        // Empty but unusable for I/O
        if position >= 0xFF4C && position < 0xFF80 {
            println!("Attempt to write at position {:X}", position);
            return;
        }

        
        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            self.internal_ram.write_8(position - 0xFF80, value);
            return;
        }

        // Interrupt enable
        if position == 0xFFFF {
            self.interrupt_enable.from_u8(value);
            return;
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }

    pub fn write_16(&mut self, position: u16, value: u16) {
        // ROM
        if position < 0x8000 {
            println!("Attempt to write at Memory {:X}. ROM is not writable!!!", position);
            return;
        }

        if position >= 0x8000 && position < 0xA000 {
            return self.video_ram.write_16(position - 0x8000, value);
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.write_16(position - 0xC000, value);
        }

        if position >= 0xE000 && position < 0xFE00 {
            return self.internal_ram_8k.write_16(position - 0xE000, value);
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.write_16(position - 0xFF80, value);
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }

    pub fn scx(&self) -> u8 {
        self.read_8(0xFF43)
    }

    pub fn scy(&self) -> u8 {
        self.read_8(0xFF42)
    }

    pub fn bgp(&self) -> u8 {
        self.read_8(0xFF47)
    }
}