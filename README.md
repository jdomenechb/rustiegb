# RustieGB
GB emulator written in Rust

## Blargg test status

### cpu_instrs
- [X] 01: Special
- [x] 02: Interrupts
- [x] 03: op sp,hl
- [x] 04: op r,imm
- [x] 05: op rp
- [x] 06: ld r,r
- [x] 07: jr,jp,call,ret,rst
- [x] 08: misc instrs
- [x] 09: op r,r
- [x] 10: bit ops
- [x] 11: op a,(hl)

### instr_timing
- [x] Instruction timing

### mem_timing
- [ ] 01: read_timing
- [ ] 02: write_timing
- [ ] 03: modify_timing

### mem_timing2
- [ ] 01: read_timing
- [ ] 02: write_timing
- [ ] 03: modify_timing

### oam_bug
- [ ] 01: LCD sync
- [ ] 02: Causes
- [ ] 03: Non causes
- [ ] 04: Scanline timing
- [ ] 05: Timing bug
- [ ] 06: Timing no bug
- [ ] 07: Timing effect
- [ ] 08: Instr Effect

### dmg_sound
- [ ] 01: Registers
- [ ] 02: Len ctr
- [ ] 03: Trigger
- [ ] 04: Sweep
- [ ] 05: Sweep details
- [ ] 06: Overflow on trigger
- [ ] 07: Len sweep period sync
- [ ] 08: Len ctr during power
- [ ] 09: Wave read while one
- [ ] 10: Wave trigger while on
- [ ] 11: Regs after power
- [ ] 12: Wave write while on
