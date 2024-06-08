use crate::Byte;

pub mod audio_registers;
mod div;
mod dma;
mod interrupt_enable;
mod interrupt_flag;
pub mod joypad;
mod lcdc;
mod ly;
pub mod registers;
mod sio_control;
pub mod stat;
mod tima;
mod timer_control;
pub mod wave_pattern_ram;

trait ResettableRegister {
    fn reset(&mut self) {}
}

trait UpdatableRegister {
    fn update(&mut self, value: Byte);
    fn reset(&mut self) {
        self.update(0);
    }
}
