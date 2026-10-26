extern crate anyhow;
extern crate cpal;
extern crate image;
extern crate piston_window;

mod audio;
mod audiold;
mod bus;
mod cartridge;
mod configuration;
mod cpu;
mod debug;
mod gpu;
mod io;
mod memory;
mod utils;

use crate::bus::cpu_memory_bus::CpuMemoryBus;
use crate::cartridge::Cartridge;
use crate::configuration::{Configuration, RuntimeConfig};
use crate::gpu::color::Color;
use crate::io::registers::IORegisters;
use crate::memory::bootstrap_rom::BootstrapRom;
use crate::piston_window::texture::UpdateTexture;
use cpu::Cpu;
use gpu::Gpu;
use image::ImageBuffer;
use io::joypad::JoypadHandler;
use memory::Memory;
use parking_lot::RwLock;
use piston_window::graphics::image as draw_image;
use piston_window::graphics::{Transformed, clear};
use piston_window::texture::{Filter, Format};
use piston_window::wgpu_graphics::{Texture, TextureSettings};
use piston_window::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

const APP_NAME: &str = "RustieGB";
const WINDOW_SIZE_MULTIPLIER: u32 = 4;
const GB_CLOCK_HZ: f64 = 4_194_304.0;

type Byte = u8;
type Word = u16;
type SignedByte = i8;

fn main() {
    let configuration = Configuration::from_command(APP_NAME);
    let runtime_config = Arc::new(RwLock::new(RuntimeConfig::default()));

    // --- Read ROM
    let bootstrap_rom =
        BootstrapRom::new_from_optional_path(configuration.bootstrap_path.as_deref());

    let cartridge = Cartridge::new_from_path(configuration.rom_file.as_str());

    if configuration.debug_header {
        cartridge.print_header();
    }

    let window_title = format!("{} - {}", cartridge.header.title, APP_NAME);

    // --- Setting up GB components
    let io_registers = Arc::new(RwLock::new(IORegisters::default()));
    let memory = Arc::new(RwLock::new(Memory::new(
        io_registers.clone(),
        cartridge,
        bootstrap_rom,
    )));
    let joypad_handler = JoypadHandler::new(io_registers.clone(), runtime_config.clone());

    let canvas = Arc::new(RwLock::new(ImageBuffer::new(
        Gpu::PIXEL_WIDTH as u32,
        Gpu::PIXEL_HEIGHT as u32,
    )));

    let memory_thread = memory.clone();
    let io_registers_thread = io_registers.clone();
    let canvas_thread = canvas.clone();
    let runtime_config_thread = runtime_config.clone();

    std::thread::spawn(move || {
        let mut cpu = Cpu::new(
            CpuMemoryBus::new(
                memory_thread.clone(),
                Gpu::new(
                    memory_thread.clone(),
                    io_registers_thread.clone(),
                    canvas_thread.clone(),
                ),
            ),
            io_registers_thread.clone(),
            configuration.bootstrap_path.is_some(),
        );

        //let audio_unit_output = CpalAudioUnitOutput::new();

        //let mut audio_unit = AudioUnit::new(audio_unit_output, io_registers_thread.clone());

        let frame_duration =
            Duration::from_secs_f64(Cpu::AVAILABLE_CCYCLES_PER_FRAME as f64 / GB_CLOCK_HZ);
        let mut next_frame = Instant::now();

        'main_loop: loop {
            // Emulation is paced by wall-clock time, independent of the window's frame rate
            next_frame += frame_duration;
            let now = Instant::now();

            if next_frame > now {
                std::thread::sleep(next_frame - now);
            } else if now - next_frame > Duration::from_millis(100) {
                next_frame = now;
            }

            runtime_config_thread.write().reset_available_ccycles();

            while runtime_config_thread.read().cpu_has_available_ccycles() {
                if runtime_config_thread.read().has_been_reset() {
                    let mut rcw = runtime_config_thread.write();

                    cpu.reset();
                    rcw.reset_available_ccycles();
                    rcw.set_reset(false);

                    continue 'main_loop;
                }

                let debug = runtime_config_thread.read().is_debug();
                let last_instruction_cycles = cpu.next_instruction(debug, configuration.trace);

                {
                    runtime_config_thread.write().available_cycles -=
                        last_instruction_cycles as i32;
                }

                let muted = { runtime_config_thread.read().muted };

                //audio_unit.step(last_instruction_cycles, muted);
            }
        }
    });

    // --- Seting up window
    let mut window: PistonWindow = WindowSettings::new(
        window_title,
        [
            Gpu::PIXEL_WIDTH as u32 * WINDOW_SIZE_MULTIPLIER,
            Gpu::PIXEL_HEIGHT as u32 * WINDOW_SIZE_MULTIPLIER,
        ],
    )
    .exit_on_esc(true)
    .resizable(false)
    .build()
    .unwrap();

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(60);
    window.events.set_event_settings(event_settings);

    let mut texture_context = window.create_texture_context();

    let texture_settings = &mut TextureSettings::new();
    texture_settings.set_filter(Filter::Nearest);

    let mut texture: G2dTexture =
        Texture::from_image(&mut texture_context, &canvas.read(), texture_settings).unwrap();

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            joypad_handler.press(key);
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            joypad_handler.release(key);
        };

        // Actions to do on render
        event.render(|render_args| {
            let canvas_read = canvas.read();
            let (w, h) = canvas_read.dimensions();

            texture
                .update(
                    &mut texture_context,
                    Format::Rgba8,
                    &canvas_read,
                    [0, 0],
                    [w, h],
                )
                .unwrap();

            let pixel_size: (f64, f64) = (
                render_args.window_size.first().unwrap() / (Gpu::PIXEL_WIDTH as f64),
                render_args.window_size.get(1).unwrap() / (Gpu::PIXEL_HEIGHT as f64),
            );

            window.draw_2d(&event, |context, graphics, _device| {
                clear(Color::white().to_f_rgba(), graphics);

                if !io_registers.read().lcdc.lcd_control_operation {
                    return;
                }

                draw_image(
                    &texture,
                    context.transform.scale(pixel_size.0, pixel_size.1),
                    graphics,
                );
            });
        });
    }
}
