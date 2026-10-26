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
