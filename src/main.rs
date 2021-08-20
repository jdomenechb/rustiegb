mod audio;
mod cartridge;
mod configuration;
mod cpu;
mod gpu;
mod joypad;
mod math;
mod memory;

extern crate anyhow;
extern crate cpal;
extern crate image;
extern crate piston_window;

use crate::audio::audio_unit_output::{AudioUnitOutput, CpalAudioUnitOutput, DebugAudioUnitOutput};
use crate::audio::AudioUnit;
use crate::cartridge::Cartridge;
use crate::configuration::{Configuration, RuntimeConfig};
use crate::gpu::color::Color;
use crate::joypad::JoypadHandler;
use cpu::cpu::CPU;
use gpu::gpu::GPU;
use image::ImageBuffer;
use memory::memory::Memory;
use parking_lot::RwLock;
use piston_window::*;
use std::sync::{mpsc, Arc};

const APP_NAME: &str = "RustieGB";

type Byte = u8;
type Word = u16;
type SignedByte = i8;

fn main() {
    let configuration = Configuration::from_command(APP_NAME);
    let runtime_config = Arc::new(RwLock::new(RuntimeConfig::default()));

    // --- Read ROM
    let cartridge = Cartridge::new_from_path(configuration.rom_file.as_str());

    if configuration.debug_header {
        cartridge.print_header();
    }

    let window_title = format!("{} - {}", cartridge.header.title, APP_NAME);

    // --- Setting up GB components
    let memory = Arc::new(RwLock::new(Memory::new(cartridge, configuration.bootstrap)));
    let joypad_handler = JoypadHandler::new(memory.clone(), runtime_config.clone());

    let canvas = Arc::new(RwLock::new(ImageBuffer::new(
        GPU::PIXEL_WIDTH as u32,
        GPU::PIXEL_HEIGHT as u32,
    )));

    let memory_thread = memory.clone();
    let canvas_thread = canvas.clone();
    let runtime_config_thread = runtime_config.clone();
    let (sx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut cpu = CPU::new(memory_thread.clone(), configuration.bootstrap);
        let mut gpu = GPU::new(memory_thread.clone());

        let audio_unit_output: Box<dyn AudioUnitOutput> = match configuration.debug_audio {
            true => Box::new(DebugAudioUnitOutput {}),
            false => Box::new(CpalAudioUnitOutput::new()),
        };

        let mut audio_unit = AudioUnit::new(audio_unit_output, memory_thread.clone());

        loop {
            while { runtime_config_thread.read().cpu_has_available_ccycles() } {
                let last_instruction_cycles = cpu.step();

                {
                    runtime_config_thread.write().available_cycles -=
                        last_instruction_cycles as i32;
                }

                let check_vblank;
                let check_lcd_stat;
                let check_timer_overflow;
                let check_joystick;

                {
                    let mut memory_thread = memory_thread.write();
                    memory_thread.step(last_instruction_cycles);

                    check_vblank = memory_thread.interrupt_enable().is_vblank()
                        && memory_thread.interrupt_flag.is_vblank();

                    check_lcd_stat = memory_thread.interrupt_enable().is_lcd_stat()
                        && memory_thread.interrupt_flag.is_lcd_stat();

                    check_timer_overflow = memory_thread.interrupt_enable().is_timer_overflow()
                        && memory_thread.interrupt_flag.is_timer_overflow();

                    check_joystick = memory_thread.interrupt_enable().is_p10_p13_transition()
                        && memory_thread.interrupt_flag.is_p10_p13_transition();
                }

                {
                    gpu.step(last_instruction_cycles, &mut canvas_thread.write());
                }

                let muted;

                {
                    muted = runtime_config_thread.read().muted;
                }

                audio_unit.step(last_instruction_cycles, muted);

                if check_vblank {
                    cpu.vblank_interrupt();

                    continue;
                }

                if check_lcd_stat {
                    cpu.lcd_stat_interrupt();

                    continue;
                }

                if check_timer_overflow {
                    cpu.timer_overflow_interrupt();

                    continue;
                }

                // TODO: Serial transfer

                if check_joystick {
                    cpu.p10_p13_transition_interrupt();

                    continue;
                }
            }

            rx.recv();
        }
    });

    // --- Seting up window
    let mut window: PistonWindow = WindowSettings::new(window_title, [640, 576])
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(60);
    window.events.set_event_settings(event_settings);

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &canvas.read(),
        &TextureSettings::new(),
    )
    .unwrap();

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            joypad_handler.press(key);
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            joypad_handler.release(key);
        };

        // Actions to do on render
        event.render(|render_args| {
            texture
                .update(&mut texture_context, &canvas.read())
                .unwrap();

            let memory = memory.read();

            let pixel_size: (f64, f64) = (
                render_args.window_size.get(0).unwrap() / (GPU::PIXEL_WIDTH as f64),
                render_args.window_size.get(1).unwrap() / (GPU::PIXEL_HEIGHT as f64),
            );

            window.draw_2d(&event, |context, graphics, device| {
                texture_context.encoder.flush(device);

                clear(Color::white().to_f_rgba(), graphics);

                if !(&memory.lcdc).lcd_control_operation() {
                    return;
                }

                image(
                    &texture,
                    context.transform.scale(pixel_size.0, pixel_size.1),
                    graphics,
                );
            });

            runtime_config.write().reset_available_ccycles();
            sx.send(1);
        });
    }
}
