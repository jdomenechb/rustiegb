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
    let mut window: PistonWindow =
        WindowSettings::new("RustieGB", [640, 576])
            .exit_on_esc(true)
            .resizable(false)
            .build()
            .unwrap();

    // Setting up CGB components
    let mut cpu = CPU::new();
    let mut memory = Memory::new("./cpu_instrs.gb");
    //let mut memory = Memory::new("./t.gb");
    let mut gpu = GPU::new();

    // Window loop
    let debug :bool = false;
    let mut i = 1;

    while let Some(event) = window.next() {
        // Keys
        // TODO

        if let Some(render_args) = event.render_args() {
            if debug && i % 1 == 0 {
                println!("{:#X?}", cpu);
                pause();
            }

            gpu.render(&mut window, &event, render_args.window_size);
            cpu.reset_available_ccycles();

            i += 1;
        }

        if let Some(u) = event.update_args() {
            while cpu.has_available_ccycles() {
                cpu.step(&mut memory);
                gpu.step(cpu.get_last_instruction_ccycles(), &mut memory);
                //sleep(Duration::from_millis(200));
            }

            gpu.update(&u);
        }
    }
}
