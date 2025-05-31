use crate::Word;

pub struct Address {}

impl Address {
    pub const BOOTSTRAP_ROM_START: Word = 0x0001;
    pub const CARTRIDGE_START: Word = 0x0100;
    pub const IO_REGISTERS_START: Word = 0xFF00;
    pub const IO_REGISTERS_END: Word = 0xFF4B;

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

    pub const NR20_SOUND_2_UNUSED: Word = 0xFF15;
    pub const NR21_SOUND_2_WAVE_PATTERN_DUTY: Word = 0xFF16;
    pub const NR22_SOUND_2_ENVELOPE: Word = 0xFF17;
    pub const NR23_SOUND_2_FR_LO: Word = 0xFF18;
    pub const NR24_SOUND_2_FR_HI: Word = 0xFF19;

    pub const NR30_SOUND_3_ON_OFF: Word = 0xFF1A;
    pub const NR31_SOUND_3_LENGTH: Word = 0xFF1B;
    pub const NR32_SOUND_3_OUTPUT_LEVEL: Word = 0xFF1C;
    pub const NR33_SOUND_3_FR_LO: Word = 0xFF1D;
    pub const NR34_SOUND_3_FR_HI: Word = 0xFF1E;

    pub const NR40_SOUND_4_UNUSED: Word = 0xFF1F;
    pub const NR41_SOUND_4_LENGTH: Word = 0xFF20;
    pub const NR42_SOUND_4_ENVELOPE: Word = 0xFF21;
    pub const NR43_SOUND_4_FR_RANDOMNESS: Word = 0xFF22;
    pub const NR44_SOUND_4_CONTROL: Word = 0xFF23;

    pub const NR50: Word = 0xFF24;
    pub const NR51: Word = 0xFF25;
    pub const NR52_SOUND: Word = 0xFF26;

    pub const UNUSED_FF27: Word = 0xFF27;
    pub const UNUSED_FF2F: Word = 0xFF2F;
    
    pub const WAVE_PATTERN_START: Word = 0xFF30;
    pub const WAVE_PATTERN_END: Word = 0xFF3F;

    pub const LCDC: Word = 0xFF40;
    pub const STAT: Word = 0xFF41;
    pub const SCY_SCROLL_Y: Word = 0xFF42;
    pub const SCX_SCROLL_X: Word = 0xFF43;
    pub const DMA: Word = 0xFF46;
    pub const BGP_BG_WIN_PALETTE: Word = 0xFF47;
    pub const OBP1_OBJ_PALETTE: Word = 0xFF48;
    pub const OBP2_OBJ_PALETTE: Word = 0xFF49;
    pub const WY_WINDOW_Y_POSITION: Word = 0xFF4A;
    pub const WX_WINDOW_X_POSITION: Word = 0xFF4B;
    pub const IE_INTERRUPT_ENABLE: Word = 0xFFFF;
}
