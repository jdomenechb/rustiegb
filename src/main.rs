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
use clap::{App, Arg};

fn pause() {
    let mut to_discard = String::new();
    let _ = io::stdin().read_line(&mut to_discard);
}

fn main() {
    let app_name = "RustieGB";

    let matches = App::new(app_name)
        .arg(
            Arg::with_name("debug-cpu")
                .long("debug-cpu")
                .help("Prints CPU instructions on command line")
        )
        .get_matches();


    // --- Other vars
    let debug_frame_by_frame:bool = false;
    let debug_cpu :bool = matches.is_present("debug-cpu");
    let bootstrap = true;
    let mut i = 1;

    // --- Setting up GB components
    let mut cpu = CPU::new(debug_cpu, bootstrap);
    let mut memory = Memory::new("./cpu_instrs.gb", bootstrap);
    let mut gpu = GPU::new();

    // --- Seting up window
    let mut window: PistonWindow =
        WindowSettings::new(app_name, [640, 576])
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
            if debug_frame_by_frame && i % 1 == 0 {
                println!("{:#X?}", cpu);
                pause();
            }

            gpu.render(&mut window, &event, render_args.window_size, &memory);
            cpu.reset_available_ccycles();

            i += 1;
        });

        // Actions to do on update
        event.update(|update_args| {
            while cpu.has_available_ccycles() {
                cpu.step(&mut memory);
                gpu.step(cpu.get_last_instruction_ccycles(), &mut memory);
            }
        });
    }
}