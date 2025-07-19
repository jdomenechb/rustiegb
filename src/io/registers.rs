use crate::audio::apu::Apu;
use crate::bus::address::Address;
use crate::debug::{
    DebugReason, Debuggable, IO_READ_WATCHPOINTS, IO_WRITE_WATCHPOINTS, OutputDebug,
};
use crate::io::div::Div;
use crate::io::dma::Dma;
use crate::io::interrupt_enable::InterruptEnable;
use crate::io::interrupt_flag::InterruptFlag;
use crate::io::joypad::Joypad;
use crate::io::lcdc::Lcdc;
use crate::io::ly::LY;
use crate::io::sio_control::SioControl;
use crate::io::stat::{STATMode, Stat};
use crate::io::tima::Tima;
use crate::io::timer_control::TimerControl;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::{Byte, Word};
use std::collections::BTreeMap;

pub struct IORegisters {
    pub p1: Joypad,
    serial_transfer_data: Byte,
    sio_control: SioControl,
    pub div: Div,
    pub tima: Tima,
    pub tma: Byte,
    pub timer_control: TimerControl,
    pub interrupt_flag: InterruptFlag,

    pub apu: Apu,

    // Wave pattern ram (FF30 - FF3F)
    pub wave_pattern_ram: WavePatternRam,
    pub lcdc: Lcdc,
    pub stat: Stat,
    pub scy: Byte,
    pub scx: Byte,
    // FF44
    pub ly: LY,
    // FF45
    pub lyc: Byte,
    pub dma: Dma,
    pub bgp: Byte,
    pub obp1: Byte,
    pub obp2: Byte,
    pub wy: Byte,
    pub wx: Byte,

    pub interrupt_enable: InterruptEnable,
}

impl IORegisters {
    pub fn step(&mut self, last_instruction_cycles: u8) -> Option<Word> {
        let mut to_return = None;

        if self.dma.step(last_instruction_cycles) {
            to_return = Some(Word::from(&self.dma));
        }

        self.div.step(last_instruction_cycles);

        if !self.timer_control.started {
            self.tima.reset_cycles();
            return to_return;
        }

        let tima_cycles_overflowed = self
            .tima
            .step(last_instruction_cycles, self.timer_control.get_divider());

        if tima_cycles_overflowed {
            self.interrupt_flag.set_timer_overflow(true);
            self.tima.value = self.tma;
        }

        self.apu.step();

        to_return
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

                self.interrupt_flag.set_vblank(true);
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

    fn determine_ly_interrupt(&mut self) {
        let ly = self.ly.value;

        let new_value = ly == self.lyc;

        self.stat.coincidence_flag = new_value;

        if self.stat.lyc_ly_coincidence && new_value {
            self.interrupt_flag.set_lcd_stat(true);
        }
    }

    pub fn ly_increment(&mut self) {
        self.ly.increment();
        self.determine_ly_interrupt();
    }

    pub fn ly_reset(&mut self) {
        self.ly.reset();
        self.determine_ly_interrupt();
    }

    pub fn ly_reset_wo_interrupt(&mut self) {
        self.ly.reset();
    }
}

impl Debuggable for IORegisters {
    fn get_debug_values(&self) -> BTreeMap<&str, String> {
        self.apu.get_debug_values()
    }
}

impl Default for IORegisters {
    fn default() -> Self {
        Self {
            p1: Joypad::new(),
            serial_transfer_data: 0,
            sio_control: SioControl::default(),
            div: Div::default(),
            tima: Tima::default(),
            tma: 0,
            timer_control: TimerControl::default(),
            interrupt_flag: InterruptFlag::new(),

            apu: Apu::default(),

            wave_pattern_ram: WavePatternRam::default(),
            lcdc: Lcdc::default(),
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
            interrupt_enable: InterruptEnable::default(),
        }
    }
}

impl ReadMemory for IORegisters {
    fn read_byte(&self, position: Word) -> Byte {
        let mut output_debug = OutputDebug::new_with_reason(DebugReason::IORead(position));
        let debug_watchpoint = IO_READ_WATCHPOINTS.contains(&position);

        if debug_watchpoint {
            output_debug.push_situation("Content", self.get_debug_values());
            output_debug.print();
        }

        match position {
            Address::P1_JOYPAD => self.p1.to_byte(),
            Address::SB_SERIAL_TRANSFER_DATA => self.serial_transfer_data,
            Address::SC_SIO_CONTROL => self.sio_control.value,
            Address::DIV_DIVIDER_REGISTER => self.div.value,
            Address::TIMA_TIMER_COUNTER => self.tima.value,
            Address::TMA_TIMER_MODULO => self.tma,
            Address::IF_INTERRUPT_FLAG => (&self.interrupt_flag).into(),

            Address::APU_START..=Address::APU_END => self.apu.read_byte(position),
            Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END => self
                .wave_pattern_ram
                .read_byte(position - Address::WAVE_PATTERN_START),
            Address::LCDC => (&self.lcdc).into(),
            Address::STAT => (&self.stat).into(),
            Address::SCY_SCROLL_Y => self.scy,
            Address::SCX_SCROLL_X => self.scx,
            0xFF44 => self.ly.value,
            0xFF45 => self.lyc,
            Address::DMA => self.dma.value,
            Address::BGP_BG_WIN_PALETTE => self.bgp,
            Address::OBP1_OBJ_PALETTE => self.obp1,
            Address::OBP2_OBJ_PALETTE => self.obp2,
            Address::WY_WINDOW_Y_POSITION => self.wy,
            Address::WX_WINDOW_X_POSITION => self.wx,
            Address::IE_INTERRUPT_ENABLE => self.interrupt_enable.value,

            _ => {
                println!("Read address {position:X} not supported for IORegisters");
                0xFF
            }
        }
    }
}

impl WriteMemory for IORegisters {
    fn write_byte(&mut self, position: Word, value: Byte) {
        let mut output_debug = OutputDebug::new_with_reason(DebugReason::IOWrite(position, value));
        let debug_watchpoint = IO_WRITE_WATCHPOINTS.contains(&position);

        if debug_watchpoint {
            output_debug.push_situation("Before", self.get_debug_values());
        }

        match position {
            Address::P1_JOYPAD => self.p1.parse_byte(value),
            Address::SB_SERIAL_TRANSFER_DATA => self.serial_transfer_data = value,
            Address::SC_SIO_CONTROL => self.sio_control.update(value),
            Address::UNUSED_FF03 => {
                println!("Attempt to write at an unused RAM position {position:X}",)
            }
            Address::DIV_DIVIDER_REGISTER => self.div.reset_value(),
            Address::TIMA_TIMER_COUNTER => self.tima.value = value,
            Address::TMA_TIMER_MODULO => self.tma = value,
            Address::TAC_TIMER_CONTROL => self.timer_control.update(value),
            0xFF08..=0xFF0E => {
                println!("Attempt to write at an unused RAM position {position:X}",)
            }
            Address::IF_INTERRUPT_FLAG => self.interrupt_flag.update(value),
            Address::APU_START..=Address::APU_END => self.apu.write_byte(position, value),
            0xFF30..=0xFF3F => {
                self.wave_pattern_ram.write_byte(position - 0xFF30, value);
                self.apu.audio_3_reg_written.wave_pattern = true;
            }
            Address::LCDC => self.lcdc.update(value),
            Address::STAT => self.stat.update(value),
            Address::SCY_SCROLL_Y => self.scy = value,
            Address::SCX_SCROLL_X => self.scx = value,
            0xFF44 => self.ly.value = value,
            0xFF45 => self.lyc = value,
            Address::DMA => self.dma.update(value),
            Address::BGP_BG_WIN_PALETTE => self.bgp = value,
            Address::OBP1_OBJ_PALETTE => self.obp1 = value,
            Address::OBP2_OBJ_PALETTE => self.obp2 = value,
            Address::WY_WINDOW_Y_POSITION => self.wy = value,
            Address::WX_WINDOW_X_POSITION => self.wx = value,
            Address::IE_INTERRUPT_ENABLE => self.interrupt_enable.update(value),
            Address::UNUSED_FF27..=Address::UNUSED_FF2F => {
                println!("Attempt to write at an unused RAM position {position:X}")
            }
            _ => panic!("Write address not supported for IORegisters"),
        }

        if debug_watchpoint {
            output_debug.push_situation("After", self.get_debug_values());
            output_debug.print()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_when_sound_is_turned_off_wave_pattern_register_is_writable() {
        let mut io_registers = IORegisters::default();

        io_registers.write_byte(Address::NR52_SOUND, 0);

        for position in Address::WAVE_PATTERN_START..=Address::WAVE_PATTERN_END {
            io_registers.write_byte(position, 0xFF);
            assert_eq!(0xFF, io_registers.read_byte(position));
        }
    }

    #[test]
    fn test_correct_data_when_writing_audio_registers() {
        let mut io_registers = IORegisters::default();

        // Unused registers
        for position in Address::UNUSED_FF27..=Address::UNUSED_FF2F {
            io_registers.write_byte(position, 0xFF);
            io_registers.write_byte(position, 0);

            assert_eq!(
                io_registers.read_byte(position),
                0xFF,
                "Wrong data when writing register {:X}",
                position
            );
        }

        // WAVE
        for position in 0xFF30..0xFF40 {
            io_registers.write_byte(position, 0xFF);
            io_registers.write_byte(position, 0);

            assert_eq!(
                io_registers.read_byte(position),
                0,
                "Wrong data when writing register {:X}",
                position
            );
        }
    }
}
