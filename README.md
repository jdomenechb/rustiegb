# RustieGB

GB emulator written in Rust.

This project's original intention was to learn Rust by coding something cool and, at the same time, complex. Therefore,
this emulator only features the basics to emulate GB and focus in learning to work with Rust rather than new
functionalities, speed or improvements.

## Dev environment set up
### Dependencies

```
sudo apt-get update && sudo apt-get install -y libasound2-dev
```

## Blargg test status

### CPU

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

### Instruction timing

- [x] Instruction timing

### DMG Sound

- [x] 01: Registers
- [x] 02: Len CTR
- [x] 03: Trigger
- [x] 04: Sweep
- [x] 05: Sweep details
- [x] 06: Overflow on trigger
- [x] 07: Len Sweep period sync
- [ ] 08: Len CTR during power
- [ ] 09: Wave read while on
- [ ] 10: Wave trigger while on
- [ ] 11: Regs after power
- [ ] 12: Wave write while on

### OAM Bug

- [ ] 01: LCD sync
- [ ] 02: Causes
- [x] 03: Non Causes
- [ ] 04: Scanline timing
- [ ] 05: Timing bug
- [X] 06: Timing no bug
- [ ] 07: Timing effect
- [ ] 08: Instr effect


