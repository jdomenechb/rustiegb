mod cpu;
mod memory;
mod math;
mod gpu;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use cpu::cpu::CPU;
use gpu::gpu::GPU;
use memory::memory::Memory;
use std::thread::sleep;
use std::time::Duration;
use std::io;

use opengl_graphics::{ GlGraphics, OpenGL };
use piston::window::WindowSettings;
use piston::event_loop::*;
use glutin_window::GlutinWindow as Window;
use piston::input::*;

fn pause() {
    let mut to_discard = String::new();
    let _ = io::stdin().read_line(&mut to_discard);
}


fn main() {
    // Create the Window and the OpenGL context
    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("RustieGB", [640, 576])
        .opengl(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Setting up CGB components
    let mut cpu = CPU::new();
    let mut memory = Memory::new("./cpu_instrs.gb");
    //let mut memory = Memory::new("./t.gb");
    let mut gpu = GPU::new(GlGraphics::new(opengl));

    // Window loop
    let mut events = Events::new(EventSettings::new());
    let debug :bool = false;
    let mut i = 1;

    while let Some(e) = events.next(&mut window) {
        // Keys
        // TODO

        //sleep(Duration::from_millis(50));

        if let Some(r) = e.render_args() {
            if debug && i % 1 == 0 {
                println!("{:#X?}", cpu);
                pause();
            }

            gpu.render(&r);

            i += 1;
        }

        if let Some(u) = e.update_args() {
            cpu.step(&mut memory);
            gpu.update(&u);
        }
    }
}
