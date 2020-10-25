mod cpu;
mod memory;
mod math;
mod gpu;

extern crate piston_window;

use cpu::cpu::CPU;

use gpu::gpu::GPU;
use memory::memory::Memory;
use std::thread::sleep;
use std::time::Duration;
use std::io;

use piston_window::*;

fn pause() {
    let mut to_discard = String::new();
    let _ = io::stdin().read_line(&mut to_discard);
}

fn main() {
    // --- Other vars
    let debug :bool = false;
    let bootstrap = true;
    let mut i = 1;

    // --- Setting up GB components
    let mut cpu = CPU::new();
    let mut  memory = Memory::new("./cpu_instrs.gb", bootstrap);

    if bootstrap {
        cpu.registers.pc = 0;
    }

    let mut gpu = GPU::new();

    // --- Seting up window
    let mut window: PistonWindow =
        WindowSettings::new("RustieGB", [640, 576])
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(60);
    event_settings.set_ups(60);
    window.events.set_event_settings(event_settings);

    while let Some(event) = window.next() {
        // TODO: Keys

        // Actions to do on render
        event.render(|render_args| {
            if debug && i % 1 == 0 {
                println!("{:#X?}", cpu);
                pause();
            }

            gpu.render(&mut window, &event, render_args.window_size);
            cpu.reset_available_ccycles();

            i += 1;
        });

        // Actions to do on update
        event.update(|update_args| {
            while cpu.has_available_ccycles() {
                cpu.step(&mut memory);
                gpu.step(cpu.get_last_instruction_ccycles(), &mut memory);
            }

            gpu.update(&update_args);
        });
    }
}