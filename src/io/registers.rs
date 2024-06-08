use crate::io::div::Div;
use crate::io::dma::Dma;
use crate::io::interrupt_flag::InterruptFlag;
use crate::io::joypad::Joypad;
use crate::io::lcdc::Lcdc;
use crate::io::ly::LY;
use crate::io::nr52::NR52;
use crate::io::sio_control::SioControl;
use crate::io::stat::{STATMode, Stat};
use crate::io::tima::Tima;
use crate::io::timer_control::TimerControl;
use crate::io::wave_pattern_ram::WavePatternRam;
use crate::memory::address::Address;
use crate::memory::audio_registers::AudioRegisters;
use crate::memory::interrupt_enable::InterruptEnable;
use crate::memory::memory_sector::{ReadMemory, WriteMemory};
use crate::memory::AudioRegWritten;
use crate::{Byte, Word};

pub struct IORegisters {
    pub p1: Joypad,
    serial_transfer_data: Byte,
    sio_control: SioControl,
    pub div: Div,
    pub tima: Tima,
    pub tma: Byte,
    pub timer_control: TimerControl,
    pub interrupt_flag: InterruptFlag,

    nr10: Byte,
    nr11: Byte,
    nr12: Byte,
    pub(crate) nr13: Byte,
    pub(crate) nr14: Byte,

    nr20: Byte,
    nr21: Byte,
    nr22: Byte,
    nr23: Byte,
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
    pub nr52: NR52,
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

    // --- OTHER - Out of scope of register
    // Audio
    pub audio_1_reg_written: AudioRegWritten,
    pub audio_2_reg_written: AudioRegWritten,
    pub audio_3_reg_written: AudioRegWritten,
    pub audio_4_reg_written: AudioRegWritten,
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

        to_return
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
            self.audio_1_reg_written.clone(),
            self.audio_2_reg_written.clone(),
            self.audio_3_reg_written.clone(),
            self.audio_4_reg_written.clone(),
        );

        self.audio_1_reg_written = AudioRegWritten::default();
        self.audio_2_reg_written = AudioRegWritten::default();
        self.audio_3_reg_written = AudioRegWritten::default();
        self.audio_4_reg_written = AudioRegWritten::default();

        to_return
    }

    pub fn update_audio_1_frequency(&mut self, frequency: Word) {
        self.nr13 = (frequency & 0xFF) as Byte;
        self.nr14 = (self.nr14 & 0b11111000) | ((frequency >> 8) & 0b111) as Byte;
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        let mut sweep = None;
        let start_address = match channel {
            1 => {
                sweep = Some(self.nr10);
                Address::NR14_SOUND_1_FR_HI
            }
            2 => Address::NR24_SOUND_3_FR_HI,
            3 => {
                sweep = Some(self.nr30);
                0xFF1E
            }
            4 => 0xFF23,
            _ => panic!("Invalid channel provided"),
        };

        AudioRegisters::new(
            self.read_byte(start_address),
            self.read_byte(start_address - 1),
            self.read_byte(start_address - 2),
            self.read_byte(start_address - 3),
            sweep,
        )
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
            audio_1_reg_written: AudioRegWritten::default(),
            audio_2_reg_written: AudioRegWritten::default(),
            audio_3_reg_written: AudioRegWritten::default(),
            audio_4_reg_written: AudioRegWritten::default(),
        }
    }
}

impl ReadMemory for IORegisters {
    fn read_byte(&self, position: Word) -> Byte {
        match position {
            Address::P1_JOYPAD => self.p1.to_byte(),
            Address::SB_SERIAL_TRANSFER_DATA => self.serial_transfer_data,
            Address::SC_SIO_CONTROL => self.sio_control.value,
            Address::UNUSED_FF03 => 0xFF,
            Address::DIV_DIVIDER_REGISTER => self.div.value,
            Address::TIMA_TIMER_COUNTER => self.tima.value,
            Address::TMA_TIMER_MODULO => self.tma,
            0xFF08..=0xFF0E => 0xFF,
            Address::IF_INTERRUPT_FLAG => (&self.interrupt_flag).into(),
            Address::NR10_SOUND_1_SWEEP => self.nr10,
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => self.nr11,
            Address::NR12_SOUND_1_ENVELOPE => self.nr12,
            Address::NR13_SOUND_1_FR_LO => self.nr13,
            Address::NR14_SOUND_1_FR_HI => self.nr14,
            Address::NR20_SOUND_2_UNUSED => self.nr20,
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => self.nr21,
            Address::NR22_SOUND_2_ENVELOPE => self.nr22,
            Address::NR23_SOUND_2_FR_LO => self.nr23,
            Address::NR24_SOUND_3_FR_HI => self.nr24,
            0xFF1A => self.nr30,
            0xFF1B => self.nr31,
            0xFF1C => self.nr32,
            0xFF1D => self.nr33,
            0xFF1E => self.nr34,
            0xFF1F => self.nr40,
            0xFF20 => self.nr41,
            0xFF21 => self.nr42,
            0xFF22 => self.nr43,
            0xFF23 => self.nr44,
            0xFF24 => self.nr50,
            0xFF25 => self.nr51,
            Address::NR52_SOUND => self.nr52.value,
            0xFF30..=0xFF3F => self.wave_pattern_ram.read_byte(position - 0xFF30),
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
            _ => panic!("Read address not supported for IORegisters"),
        }
    }
}

impl WriteMemory for IORegisters {
    fn write_byte(&mut self, position: Word, value: Byte) {
        match position {
            Address::P1_JOYPAD => self.p1.parse_byte(value),
            Address::SB_SERIAL_TRANSFER_DATA => self.serial_transfer_data = value,
            Address::SC_SIO_CONTROL => self.sio_control.update(value),
            Address::UNUSED_FF03 => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            Address::DIV_DIVIDER_REGISTER => self.div.reset_value(),
            Address::TIMA_TIMER_COUNTER => self.tima.value = value,
            Address::TMA_TIMER_MODULO => self.tma = value,
            Address::TAC_TIMER_CONTROL => self.timer_control.update(value),
            0xFF08..=0xFF0E => {
                println!("Attempt to write at an unused RAM position {:X}", position)
            }
            Address::IF_INTERRUPT_FLAG => self.interrupt_flag.update(value),
            Address::NR10_SOUND_1_SWEEP => {
                if self.nr52.is_on() {
                    self.nr10 = value;
                    self.audio_1_reg_written.sweep_or_wave_onoff = true;
                }
            }
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr11 = value;
                    self.audio_1_reg_written.length = true;
                }
            }
            Address::NR12_SOUND_1_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr12 = value;
                    self.audio_1_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR13_SOUND_1_FR_LO => {
                if self.nr52.is_on() {
                    self.nr13 = value;
                    self.audio_1_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR14_SOUND_1_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_1_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_channel_active(1);
                }

                self.nr14 = value;
            }
            Address::NR20_SOUND_2_UNUSED => {
                if self.nr52.is_on() {
                    self.nr20 = value;
                }
            }
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr21 = value;
                    self.audio_2_reg_written.length = true;
                }
            }
            Address::NR22_SOUND_2_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr22 = value;
                    self.audio_2_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR23_SOUND_2_FR_LO => {
                if self.nr52.is_on() {
                    self.nr23 = value;
                    self.audio_2_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR24_SOUND_3_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_2_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_channel_active(2);
                }

                self.nr24 = value;
            }
            0xFF1A => {
                if self.nr52.is_on() {
                    self.nr30 = value;
                    self.audio_3_reg_written.sweep_or_wave_onoff = true;
                }
            }
            0xFF1B => {
                if self.nr52.is_on() {
                    self.nr31 = value;
                    self.audio_3_reg_written.length = true;
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
                    self.audio_3_reg_written.frequency_or_poly_counter = true;
                }
            }
            0xFF1E => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_3_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
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
                    self.audio_4_reg_written.length = true;
                }
            }
            0xFF21 => {
                if self.nr52.is_on() {
                    self.nr42 = value;
                    self.audio_4_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            0xFF22 => {
                if self.nr52.is_on() {
                    self.nr43 = value;
                    self.audio_4_reg_written.frequency_or_poly_counter = true;
                }
            }
            0xFF23 => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_4_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
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
            Address::NR52_SOUND => {
                self.nr52.value = value & 0b10000000;

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
            0xFF30..=0xFF3F => {
                self.wave_pattern_ram.write_byte(position - 0xFF30, value);
                self.audio_3_reg_written.wave_pattern = true;
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
            _ => panic!("Write address not supported for IORegisters"),
        }
    }
}
