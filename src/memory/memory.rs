use super::read_only_memory_sector::ReadOnlyMemorySector;
use super::internal_ram_memory_sector::InternalRamMemorySector;
use super::internal_ram_8k_memory_sector::InternalRam8kMemorySector;
use super::video_ram_8k_memory_sector::VideoRam8kMemorySector;
use super::timer_control::TimerControl;
use super::interrupt_flag::InterruptFlag;
use super::lcdc::LCDC;
use std::fs::File;
use std::io::Read;

pub struct Memory {
    rom: ReadOnlyMemorySector,
    video_ram: VideoRam8kMemorySector,
    internal_ram_8k: InternalRam8kMemorySector,
    // FF07
    timer_control: TimerControl,
    // FF0F
    interrupt_flag: InterruptFlag,
    // FF24
    nr50: u8,
    // FF25
    nr51: u8,
    // FF26
    nr52: u8,
    // FF40
    lcdc: LCDC,
    // FF42 - FF43
    scy: u8, 
    scx: u8,
    // FF44
    ly: u8,
    // FF47
    bgp: u8,
    // FF80 - FFFE
    internal_ram: InternalRamMemorySector,
    // FFFF
    interrupt_enable: InterruptFlag,
}

impl Memory {
    pub fn new(rom_path : &str) -> Memory {
        let mut data:Vec<u8> = Vec::with_capacity(0x8000);
        let mut rom_file = File::open(rom_path).expect("file not found");
        rom_file.read_to_end(&mut data).expect("Error on reading ROM contents");

        return Memory {
            rom: ReadOnlyMemorySector::new(data),
            video_ram: VideoRam8kMemorySector::new(),
            internal_ram_8k: InternalRam8kMemorySector::new(),
            timer_control: TimerControl::new(),
            interrupt_flag: InterruptFlag::new(),
            nr50: 0x77,
            nr51: 0xf3,
            nr52: 0xf1,
            lcdc: LCDC::new(),
            scy: 0,
            scx: 0,
            ly: 0,
            bgp: 0xFC,
            internal_ram: InternalRamMemorySector::new(),
            interrupt_enable: InterruptFlag::new()
        };
    }

    pub fn read_8(&self, position: u16) -> u8 {
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

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_8(position - 0xFF80);
        }

        panic!("ERROR: Memory address {:X} not readable", position);

        return 0;
    }

    pub fn read_8_signed(&self, position: u16) -> i8 {
        let value :u8 = self.read_8(position);

        return value as i8;
    }

    pub fn read_16(&self, position: u16) -> u16 {
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

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.read_16(position - 0xFF80);
        }

        panic!("ERROR: Memory address {:X} not readable", position);

        return 0;
    }

    pub fn write_8(&mut self, position: u16, value: u8) {
        // ROM
        if position < 0x8000 {
            panic!("ROM is not writable!!!");
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

        // LY
        if position == 0xFF47 {
            self.bgp = value;
            return;
        }

        // Empty but unusable for I/O
        if position >= 0xFF4C && position < 0xFF80 {
            println!("Attmept to write at position {:X}", position);
            return;
        }

        
        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            self.internal_ram.write_8(position - 0xFF80, value);
            return;
        }

        if position == 0xFFFF {
            self.interrupt_enable.from_u8(value);
            return;
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }

    pub fn write_16(&mut self, position: u16, value: u16) {
        // ROM
        if position < 0x8000 {
            panic!("ROM is not writable!!!");
        }

        // Internal RAM 8k
        if position >= 0xC000 && position < 0xE000 {
            return self.internal_ram_8k.write_16(position - 0xC000, value);
        }

        // Internal RAM
        if position >= 0xFF80 && position < 0xFFFF {
            return self.internal_ram.write_16(position - 0xFF80, value);
        }

        panic!("ERROR: Memory address {:X} not writable", position);
    }
}