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

    pub fn read_signed_byte(&self, position: Word) -> SignedByte {
        self.memory.read().read_byte(position) as SignedByte
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

    pub fn step(&mut self, cycles: u8) {
        let cycles = cycles - self.cycles_accumulated;
        self.cycles_accumulated = 0;
        {
            self.memory.write().step(cycles);
        }

        self.gpu.step(cycles);
    }
}

impl ReadMemory for CpuMemoryBus {
    fn read_byte(&self, position: Word) -> Byte {
        self.memory.read().read_byte(position)
    }
}

impl WriteMemory for CpuMemoryBus {
    fn write_byte(&mut self, position: Word, value: Byte) {
        self.memory.write().write_byte(position, value)
    }
}

impl ReadWordMemory for CpuMemoryBus {
    fn read_word(&self, position: Word) -> Word {
        self.memory.read().read_word(position)
    }
}

impl WriteWordMemory for CpuMemoryBus {
    fn write_word(&mut self, position: Word, value: Word) {
        self.memory.write().write_word(position, value)
    }
}
