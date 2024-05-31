use crate::Word;

pub struct Address {}

impl Address {
    pub const BOOTSTRAP_ROM_START: Word = 0x0001;
    pub const CARTRIDGE_START: Word = 0x0100;
    pub const IO_REGISTERS_START: Word = 0xFF00;

    /*
       I/O Registers
    */
    pub const P1_JOYPAD: Word = 0xFF00;
    pub const SB_SERIAL_TRANSFER_DATA: Word = 0xFF01;
    pub const SC_SIO_CONTROL: Word = 0xFF02;
    pub const UNUSED_FF03: Word = 0xFF03;
    pub const DIV_DIVIDER_REGISTER: Word = 0xFF04;
    pub const TIMA_TIMER_COUNTER: Word = 0xFF05;
    pub const TMA_TIMER_MODULO: Word = 0xFF06;
    pub const TAC_TIMER_CONTROL: Word = 0xFF07;
    pub const IF_INTERRUPT_FLAG: Word = 0xFF0F;
    pub const NR10_SOUND_1_SWEEP: Word = 0xFF10;
    pub const NR11_SOUND_1_WAVE_PATTERN_DUTY: Word = 0xFF11;
    pub const NR12_SOUND_1_ENVELOPE: Word = 0xFF12;
    pub const NR13_SOUND_1_FR_LO: Word = 0xFF13;
    pub const NR14_SOUND_1_FR_HI: Word = 0xFF14;
    pub const NR52_SOUND: Word = 0xFF26;
    pub const STAT: Word = 0xFF41;
    pub const DMA: Word = 0xFF46;
    pub const IE_INTERRUPT_ENABLE: Word = 0xFFFF;
}
