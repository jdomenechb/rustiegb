use crate::bus::address::Address;
use crate::io::audio_registers::nr52::NR52;
use crate::io::audio_registers::nrxx::NRxx;
use crate::io::audio_registers::AudioRegWritten;
use crate::io::audio_registers::AudioRegisters;
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

pub struct IORegisters {
    pub p1: Joypad,
    serial_transfer_data: Byte,
    sio_control: SioControl,
    pub div: Div,
    pub tima: Tima,
    pub tma: Byte,
    pub timer_control: TimerControl,
    pub interrupt_flag: InterruptFlag,

    nr10: NRxx,
    nr11: NRxx,
    nr12: NRxx,
    nr13: NRxx,
    nr14: NRxx,

    nr21: NRxx,
    nr22: NRxx,
    nr23: NRxx,
    nr24: NRxx,

    nr30: NRxx,
    nr31: NRxx,
    nr32: NRxx,
    nr33: NRxx,
    nr34: NRxx,

    nr41: NRxx,
    nr42: NRxx,
    nr43: NRxx,
    nr44: NRxx,

    nr50: Byte,
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
        self.nr13.update_low_frequency(frequency);
        self.nr14.update_high_frequency(frequency)
    }

    pub fn read_audio_registers(&self, channel: u8) -> AudioRegisters {
        match channel {
            1 => AudioRegisters::new(
                self.nr14.value,
                self.nr13.value,
                self.nr12.value,
                self.nr11.value,
                Some(self.nr10.value),
            ),
            2 => AudioRegisters::new(
                self.nr24.value,
                self.nr23.value,
                self.nr22.value,
                self.nr21.value,
                None,
            ),
            3 => AudioRegisters::new(
                self.nr34.value,
                self.nr33.value,
                self.nr32.value,
                self.nr31.value,
                Some(self.nr30.value),
            ),
            4 => AudioRegisters::new(
                self.nr44.value,
                self.nr43.value,
                self.nr42.value,
                self.nr41.value,
                None,
            ),
            _ => panic!("Invalid channel provided"),
        }
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

            nr10: NRxx::new_with_used_bits(0x80, 0b0111_1111),
            nr11: NRxx::new_with_read_ored_bits(0xBF, 0x3F),
            nr12: NRxx::new(0xF3),
            nr13: NRxx::new_with_read_ored_bits(0xFF, 0xFF),
            nr14: NRxx::new_with(0xBF, 0b1100_0111, 0xBF),

            nr21: NRxx::new_with_read_ored_bits(0x3F, 0x3F),
            nr22: NRxx::new(0x00),
            nr23: NRxx::new_with_read_ored_bits(0xFF, 0xFF),
            nr24: NRxx::new_with(0xBF, 0b1100_0111, 0xBF),

            nr30: NRxx::new_with_used_bits(0x7F, 0b1000_0000),
            nr31: NRxx::new_with_read_ored_bits(0xFF, 0xFF),
            nr32: NRxx::new_with_used_bits(0x9F, 0b0110_0000),
            nr33: NRxx::new_with_read_ored_bits(0xFF, 0xFF),
            nr34: NRxx::new_with(0xBF, 0b1100_0111, 0xBF),

            nr41: NRxx::new_with(0xFF, 0b0011_1111, 0xFF),
            nr42: NRxx::new(0x00),
            nr43: NRxx::new(0x00),
            nr44: NRxx::new_with(0xBF, 0b1100_0000, 0xBF),

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

            Address::NR10_SOUND_1_SWEEP => self.nr10.read(),
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => self.nr11.read(),
            Address::NR12_SOUND_1_ENVELOPE => self.nr12.read(),
            Address::NR13_SOUND_1_FR_LO => self.nr13.read(),
            Address::NR14_SOUND_1_FR_HI => self.nr14.read(),

            Address::NR20_SOUND_2_UNUSED => 0xFF,
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => self.nr21.read(),
            Address::NR22_SOUND_2_ENVELOPE => self.nr22.read(),
            Address::NR23_SOUND_2_FR_LO => self.nr23.read(),
            Address::NR24_SOUND_2_FR_HI => self.nr24.read(),

            Address::NR30_SOUND_3_ON_OFF => self.nr30.read(),
            Address::NR31_SOUND_3_LENGTH => self.nr31.read(),
            Address::NR32_SOUND_3_OUTPUT_LEVEL => self.nr32.read(),
            Address::NR33_SOUND_3_FR_LO => self.nr33.read(),
            Address::NR34_SOUND_3_FR_HI => self.nr34.read(),

            Address::NR40_SOUND_4_UNUSED => 0xFF,
            Address::NR41_SOUND_4_LENGTH => self.nr41.read(),
            Address::NR42_SOUND_4_ENVELOPE => self.nr42.read(),
            Address::NR43_SOUND_4_FR_RANDOMNESS => self.nr43.read(),
            Address::NR44_SOUND_4_CONTROL => self.nr44.read(),

            Address::NR50 => self.nr50,
            Address::NR51 => self.nr51,
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
                    self.nr10.update(value);
                    self.audio_1_reg_written.sweep_or_wave_onoff = true;
                }
            }
            Address::NR11_SOUND_1_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr11.update(value);
                    self.audio_1_reg_written.length = true;
                }
            }
            Address::NR12_SOUND_1_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr12.update(value);
                    self.audio_1_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR13_SOUND_1_FR_LO => {
                if self.nr52.is_on() {
                    self.nr13.update(value);
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

                self.nr14.update(value);
            }
            Address::NR21_SOUND_2_WAVE_PATTERN_DUTY => {
                if self.nr52.is_on() {
                    self.nr21.update(value);
                    self.audio_2_reg_written.length = true;
                }
            }
            Address::NR22_SOUND_2_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr22.update(value);
                    self.audio_2_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR23_SOUND_2_FR_LO => {
                if self.nr52.is_on() {
                    self.nr23.update(value);
                    self.audio_2_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR24_SOUND_2_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_2_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_channel_active(2);
                }

                self.nr24.update(value);
            }
            Address::NR30_SOUND_3_ON_OFF => {
                if self.nr52.is_on() {
                    self.nr30.update(value);
                    self.audio_3_reg_written.sweep_or_wave_onoff = true;
                }
            }
            Address::NR31_SOUND_3_LENGTH => {
                if self.nr52.is_on() {
                    self.nr31.update(value);
                    self.audio_3_reg_written.length = true;
                }
            }
            Address::NR32_SOUND_3_OUTPUT_LEVEL => {
                if self.nr52.is_on() {
                    self.nr32.update(value);
                }
            }
            Address::NR33_SOUND_3_FR_LO => {
                if self.nr52.is_on() {
                    self.nr33.update(value);
                    self.audio_3_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR34_SOUND_3_FR_HI => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_3_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_channel_active(3);
                }

                self.nr34.update(value);
            }
            Address::NR41_SOUND_4_LENGTH => {
                if self.nr52.is_on() {
                    self.nr41.update(value);
                    self.audio_4_reg_written.length = true;
                }
            }
            Address::NR42_SOUND_4_ENVELOPE => {
                if self.nr52.is_on() {
                    self.nr42.update(value);
                    self.audio_4_reg_written.envelope_or_wave_out_lvl = true;
                }
            }
            Address::NR43_SOUND_4_FR_RANDOMNESS => {
                if self.nr52.is_on() {
                    self.nr43.update(value);
                    self.audio_4_reg_written.frequency_or_poly_counter = true;
                }
            }
            Address::NR44_SOUND_4_CONTROL => {
                if !self.nr52.is_on() {
                    return;
                }

                self.audio_4_reg_written.control = true;

                if value & 0b10000000 == 0b10000000 {
                    self.nr52.set_channel_active(4);
                }

                self.nr44.update(value);
            }
            Address::NR50 => {
                if self.nr52.is_on() {
                    self.nr50 = value;
                }
            }
            Address::NR51 => {
                if self.nr52.is_on() {
                    self.nr51 = value;
                }
            }
            Address::NR52_SOUND => {
                self.nr52.update(value);

                self.nr10.reset();
                self.nr11.reset();
                self.nr12.reset();
                self.nr13.reset();
                self.nr14.reset();

                // NR20 is not used
                self.nr21.reset();
                self.nr22.reset();
                self.nr23.reset();
                self.nr24.reset();

                self.nr30.reset();
                self.nr31.reset();
                self.nr32.reset();
                self.nr33.reset();
                self.nr34.reset();

                self.nr41.reset();
                self.nr42.reset();
                self.nr43.reset();
                self.nr44.reset();

                if self.nr52.is_on() {
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
            Address::NR20_SOUND_2_UNUSED => {
                // Ignored, not used
            }
            Address::NR40_SOUND_4_UNUSED => {
                // Ignored, not used
            }
            _ => panic!("Write address not supported for IORegisters"),
        }
    }
}
