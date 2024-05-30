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

use crate::audio::audio_unit_output::CpalAudioUnitOutput;
use crate::audio::AudioUnit;
use crate::cartridge::Cartridge;
use crate::configuration::{Configuration, RuntimeConfig};
use crate::gpu::color::Color;
use crate::joypad::JoypadHandler;
use crate::memory::bootstrap_rom::BootstrapRom;
use cpu::Cpu;
use gpu::Gpu;
use image::ImageBuffer;
use memory::Memory;
use parking_lot::RwLock;
use piston_window::*;
use std::sync::{mpsc, Arc};

const APP_NAME: &str = "RustieGB";
const WINDOW_SIZE_MULTIPLIER: u32 = 4;

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
    let memory = Arc::new(RwLock::new(Memory::new(cartridge, bootstrap_rom)));
    let joypad_handler = JoypadHandler::new(memory.clone(), runtime_config.clone());

    let canvas = Arc::new(RwLock::new(ImageBuffer::new(
        Gpu::PIXEL_WIDTH as u32,
        Gpu::PIXEL_HEIGHT as u32,
    )));

    let memory_thread = memory.clone();
    let canvas_thread = canvas.clone();
    let runtime_config_thread = runtime_config.clone();
    let (sx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let mut cpu = Cpu::new(
            memory_thread.clone(),
            configuration.bootstrap_path.is_some(),
        );
        let mut gpu = Gpu::new(memory_thread.clone());

        let audio_unit_output = CpalAudioUnitOutput::new();

        let mut audio_unit = AudioUnit::new(audio_unit_output, memory_thread.clone());

        loop {
            if runtime_config_thread.read().has_been_reset() {
                let mut rcw = runtime_config_thread.write();

                cpu.reset();
                rcw.reset_available_ccycles();
                rcw.set_reset(false);
            }

            while runtime_config_thread.read().cpu_has_available_ccycles() {
                let last_instruction_cycles = cpu.step(runtime_config_thread.read().is_debug());

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

                    check_vblank = memory_thread.interrupt_enable().vblank
                        && memory_thread.interrupt_flag.vblank;

                    check_lcd_stat = memory_thread.interrupt_enable().lcd_stat
                        && memory_thread.interrupt_flag.lcd_stat;

                    check_timer_overflow = memory_thread.interrupt_enable().timer_overflow
                        && memory_thread.interrupt_flag.timer_overflow;

                    check_joystick = memory_thread.interrupt_enable().p10_13_transition
                        && memory_thread.interrupt_flag.p10_13_transition;
                }

                {
                    gpu.step(last_instruction_cycles, &mut canvas_thread.write());
                }

                let muted = { runtime_config_thread.read().muted };

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

            rx.recv().expect("Could not receive from thread");
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

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

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
            texture
                .update(&mut texture_context, &canvas.read())
                .unwrap();

            let memory = memory.read();

            let pixel_size: (f64, f64) = (
                render_args.window_size.first().unwrap() / (Gpu::PIXEL_WIDTH as f64),
                render_args.window_size.get(1).unwrap() / (Gpu::PIXEL_HEIGHT as f64),
            );

            window.draw_2d(&event, |context, graphics, device| {
                texture_context.encoder.flush(device);

                clear(Color::white().to_f_rgba(), graphics);

                if !memory.lcdc.lcd_control_operation {
                    return;
                }

                image(
                    &texture,
                    context.transform.scale(pixel_size.0, pixel_size.1),
                    graphics,
                );
            });

            runtime_config.write().reset_available_ccycles();
            sx.send(1).expect("Could not send to thread");
        });
    }
}
