mod cpu;
mod gpu;
mod math;
mod memory;

extern crate image;
extern crate piston_window;

use cpu::cpu::CPU;

use gpu::gpu::GPU;
use memory::memory::Memory;
use std::io;
use std::thread::sleep;
use std::time::Duration;

use clap::{App, Arg};
use image::ImageBuffer;
use piston_window::*;

fn main() {
    let app_name = "RustieGB";

    let matches = App::new(app_name)
        .arg(
            Arg::with_name("ROMFILE")
                .required(true)
                .index(1)
                .help("Path of the ROM file to use"),
        )
        .arg(
            Arg::with_name("debug-cpu")
                .long("debug-cpu")
                .help("Prints CPU instructions on command line"),
        )
        .arg(
            Arg::with_name("bootstrap")
                .long("bootstrap")
                .help("Uses bootstrap ROM"),
        )
        .get_matches();

    // --- Other vars
    let debug_cpu: bool = matches.is_present("debug-cpu");
    let bootstrap = matches.is_present("bootstrap");
    let mut i = 1;

    // --- Setting up GB components
    let mut cpu = CPU::new(debug_cpu, bootstrap);
    let mut memory = Memory::new(matches.value_of("ROMFILE").unwrap(), bootstrap);
    let mut gpu = GPU::new();

    // --- Seting up window
    let mut window: PistonWindow = WindowSettings::new(app_name, [640, 576])
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(60);
    event_settings.set_ups(60);
    window.events.set_event_settings(event_settings);

    let mut canvas = ImageBuffer::new(GPU::PIXEL_WIDTH as u32, GPU::PIXEL_HEIGHT as u32);
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas, &TextureSettings::new()).unwrap();

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            println!("P: {:?}", key);

            match key {
                Key::X => memory.joypad().a = true,
                Key::Z => memory.joypad().b = true,
                Key::Return => memory.joypad().start = true,
                Key::RShift => memory.joypad().select = true,
                Key::Left => memory.joypad().left = true,
                Key::Right => memory.joypad().right = true,
                Key::Up => memory.joypad().up = true,
                Key::Down => memory.joypad().down = true,
                _ => {}
            }
        };

        if let Some(Button::Keyboard(key)) = event.release_args() {
            println!("R: {:?}", key);
            match key {
                Key::X => memory.joypad().a = false,
                Key::Z => memory.joypad().b = false,
                Key::Return => memory.joypad().start = false,
                Key::RShift => memory.joypad().select = false,
                Key::Left => memory.joypad().left = false,
                Key::Right => memory.joypad().right = false,
                Key::Up => memory.joypad().up = false,
                Key::Down => memory.joypad().down = false,
                _ => {}
            }
        };

        // Actions to do on render
        event.render(|render_args| {
            texture.update(&mut texture_context, &canvas).unwrap();

            gpu.render(
                &mut window,
                &event,
                render_args.window_size,
                &memory,
                &mut texture_context,
                &texture,
            );
            cpu.reset_available_ccycles();

            i += 1;
        });

        // Actions to do on update
        event.update(|update_args| {
            while cpu.has_available_ccycles() {
                if !cpu.is_halted() {
                    cpu.step(&mut memory);
                    gpu.step(cpu.get_last_instruction_ccycles(), &mut memory, &mut canvas);
                }

                if cpu.are_interrupts_enabled() {
                    if memory.interrupt_enable().is_vblank() && memory.interrupt_flag().is_vblank()
                    {
                        cpu.vblank_interrupt(&mut memory);
                        cpu.unhalt();

                        continue;
                    }
                }
            }
        });
    }
}
