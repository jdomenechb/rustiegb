use crate::gpu::Gpu;
use crate::memory::Memory;
use crate::memory::memory_sector::{ReadMemory, ReadWordMemory, WriteMemory, WriteWordMemory};
use crate::{Byte, SignedByte, Word};
use parking_lot::RwLock;
use std::sync::Arc;

pub struct CpuMemoryBus {
    memory: Arc<RwLock<Memory>>,
    gpu: Gpu,

    cycles_accumulated: u8,
}

impl CpuMemoryBus {
    pub fn new(memory: Arc<RwLock<Memory>>, gpu: Gpu) -> Self {
        Self {
            memory,
            gpu,
            cycles_accumulated: 0,
        }
    }

    pub fn read_byte(&mut self, position: Word) -> Byte {
        let value = self.memory.read().read_byte(position);

        self.cycles_accumulated += 4;
        self.step(4);

        value
    }

    pub fn read_word(&mut self, position: Word) -> Word {
        let value = self.memory.read().read_word(position);

        self.cycles_accumulated += 8;
        self.step(8);

        value
    }

    pub fn read_signed_byte(&mut self, position: Word) -> SignedByte {
        self.read_byte(position) as SignedByte
    }

    pub fn peak_byte(&self, position: Word) -> Byte {
        self.memory.read().read_byte(position)
    }

    pub fn has_bootstrap_rom(&self) -> bool {
        self.memory.read().has_bootstrap_rom()
    }

    pub fn clear_bootstrap_rom(&mut self) {
        self.memory.write().clear_bootstrap_rom();
    }

    fn single_step(&mut self) {
        {
            self.memory.write().single_step();
        }

        self.gpu.single_step();
    }

    fn step(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.single_step();
        }
    }

    pub fn step_remaining(&mut self, cycles: u8) {
        let cycles = cycles - self.cycles_accumulated;
        self.cycles_accumulated = 0;

        self.step(cycles)
    }
}

impl WriteMemory for CpuMemoryBus {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.memory.write().write_byte(position, value)
    }
}

impl WriteWordMemory for CpuMemoryBus {
    fn write_word(&mut self, position: Word, value: Word) {
        self.memory.write().write_word(position, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::address::Address;
    use image::{ImageBuffer, Rgba};

    fn create_fixture() -> (CpuMemoryBus, Arc<RwLock<Memory>>) {
        let memory = Arc::new(RwLock::new(Memory::default()));
        let io_registers = memory.read().io_registers.clone();
        let canvas = Arc::new(RwLock::new(ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
            Gpu::PIXEL_WIDTH as u32,
            Gpu::PIXEL_HEIGHT as u32,
        )));
        let gpu = Gpu::new(memory.clone(), io_registers, canvas);

        (CpuMemoryBus::new(memory.clone(), gpu), memory)
    }

    fn div(memory: &Arc<RwLock<Memory>>) -> Byte {
        memory.read().io_registers.read().div.read()
    }

    /// Runs `reads` accesses as one instruction per `reads_per_instruction` to keep the
    /// accumulated cycles within a single instruction's range
    fn run_reads(
        fixture: &mut CpuMemoryBus,
        reads: u32,
        reads_per_instruction: u32,
        cycles_per_read: u8,
        mut read: impl FnMut(&mut CpuMemoryBus),
    ) {
        let mut done = 0;

        while done < reads {
            let count = reads_per_instruction.min(reads - done);

            for _ in 0..count {
                read(fixture);
            }

            fixture.step_remaining(count as u8 * cycles_per_read);
            done += count;
        }
    }

    #[test]
    fn it_steps_4_cycles_per_byte_read() {
        let (mut fixture, memory) = create_fixture();

        run_reads(&mut fixture, 63, 3, 4, |f| {
            f.read_byte(0xC000);
        });
        assert_eq!(div(&memory), 0);

        run_reads(&mut fixture, 1, 1, 4, |f| {
            f.read_byte(0xC000);
        });
        assert_eq!(div(&memory), 1);
    }

    #[test]
    fn it_steps_8_cycles_per_word_read() {
        let (mut fixture, memory) = create_fixture();

        run_reads(&mut fixture, 31, 3, 8, |f| {
            f.read_word(0xC000);
        });
        assert_eq!(div(&memory), 0);

        run_reads(&mut fixture, 1, 1, 8, |f| {
            f.read_word(0xC000);
        });
        assert_eq!(div(&memory), 1);
    }

    #[test]
    fn it_reads_the_signed_byte_with_the_same_timing() {
        let (mut fixture, memory) = create_fixture();
        fixture.write_byte(0xC000, 0xFF);

        assert_eq!(fixture.read_signed_byte(0xC000), -1);
        fixture.step_remaining(4);

        run_reads(&mut fixture, 63, 3, 4, |f| {
            f.read_signed_byte(0xC000);
        });
        assert_eq!(div(&memory), 1);
    }

    #[test]
    fn it_does_not_step_when_peaking_or_writing() {
        let (mut fixture, memory) = create_fixture();

        for _ in 0..100 {
            fixture.write_byte(0xC000, 0xAB);
            fixture.write_word(0xC010, 0xABCD);
            assert_eq!(fixture.peak_byte(0xC000), 0xAB);
        }

        assert_eq!(div(&memory), 0);
        assert_eq!(fixture.cycles_accumulated, 0);
    }

    #[test]
    fn it_steps_only_the_remaining_cycles_of_the_instruction() {
        let (mut fixture, memory) = create_fixture();

        fixture.read_byte(0xC000);
        assert_eq!(fixture.cycles_accumulated, 4);

        // 4 cycles already consumed by the read
        fixture.step_remaining(252);
        assert_eq!(div(&memory), 0);
        assert_eq!(fixture.cycles_accumulated, 0);

        fixture.step_remaining(4);
        assert_eq!(div(&memory), 1);
    }

    #[test]
    fn it_steps_nothing_if_the_instruction_has_no_remaining_cycles() {
        let (mut fixture, memory) = create_fixture();

        fixture.read_word(0xC000);
        fixture.step_remaining(8);

        assert_eq!(fixture.cycles_accumulated, 0);

        for _ in 0..31 {
            fixture.step_remaining(8);
        }
        assert_eq!(div(&memory), 1);
    }

    #[test]
    fn it_completes_a_dma_transfer_while_stepping() {
        let (mut fixture, _memory) = create_fixture();
        fixture.write_byte(0xC000, 0xAB);
        fixture.write_byte(0xC001, 0xCD);

        fixture.write_byte(Address::DMA, 0xC0);
        assert_eq!(fixture.peak_byte(0xFE00), 0x00);

        fixture.step_remaining(159);
        assert_eq!(fixture.peak_byte(0xFE00), 0x00);

        fixture.step_remaining(1);
        assert_eq!(fixture.peak_byte(0xFE00), 0xAB);
        assert_eq!(fixture.peak_byte(0xFE01), 0xCD);
    }

    #[test]
    fn it_clears_the_bootstrap_rom() {
        let (mut fixture, _memory) = create_fixture();

        assert!(!fixture.has_bootstrap_rom());
        fixture.clear_bootstrap_rom();
        assert!(!fixture.has_bootstrap_rom());
    }
}
