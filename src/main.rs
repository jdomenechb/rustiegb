mod audio;
mod cartridge;
mod configuration;
mod cpu;
mod gpu;
mod math;
mod memory;

extern crate anyhow;
extern crate cpal;
extern crate image;
extern crate piston_window;

use crate::audio::audio_unit_output::{AudioUnitOutput, CpalAudioUnitOutput, DebugAudioUnitOutput};
use crate::audio::AudioUnit;
use crate::cartridge::Cartridge;
use crate::configuration::Configuration;
use cpu::cpu::CPU;
use gpu::gpu::GPU;
use image::ImageBuffer;
use memory::memory::Memory;
use piston_window::*;
use std::cell::RefCell;
use std::rc::Rc;

const APP_NAME: &str = "RustieGB";

type Byte = u8;
type Word = u16;
type SignedByte = i8;

fn main() {
    let configuration = Configuration::from_command(APP_NAME);

    // --- Read ROM
    let cartridge = Cartridge::new_from_path(configuration.rom_file.as_str());

    if configuration.debug_header {
        println!("{:?}", cartridge.header);
    }

    let window_title = format!("{} - {}", cartridge.header.title, APP_NAME);

    // --- Setting up GB components
    let memory = Rc::new(RefCell::new(Memory::new(
        cartridge,
        configuration.bootstrap,
    )));

    let mut cpu = CPU::new(
        memory.clone(),
        configuration.debug_cpu,
        configuration.bootstrap,
    );

    let mut gpu = GPU::new(memory.clone());

    let audio_unit_output: Box<dyn AudioUnitOutput> = match configuration.debug_audio {
        true => Box::new(DebugAudioUnitOutput {}),
        false => Box::new(CpalAudioUnitOutput::new()),
    };

    let mut audio_unit = AudioUnit::new(audio_unit_output, memory.clone());

    // --- Seting up window
    let mut window: PistonWindow = WindowSettings::new(window_title, [640, 576])
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
            let mut memory = memory.borrow_mut();

            match key {
                Key::X => {
                    memory.joypad().a = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Z => {
                    memory.joypad().b = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Return => {
                    memory.joypad().start = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::RShift => {
                    memory.joypad().select = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Left => {
                    memory.joypad().left = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Right => {
                    memory.joypad().right = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Up => {
                    memory.joypad().up = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::Down => {
                    memory.joypad().down = true;
                    memory.interrupt_flag().set_p10_p13_transition(true);
                }
                Key::M => {
                    audio_unit.toggle_mute();
                }
                Key::Space => {
                    cpu.set_user_speed_multiplier(20);
                }
                _ => {}
            };
        }

        if let Some(Button::Keyboard(key)) = event.release_args() {
            let mut memory = memory.borrow_mut();

            match key {
                Key::X => memory.joypad().a = false,
                Key::Z => memory.joypad().b = false,
                Key::Return => memory.joypad().start = false,
                Key::RShift => memory.joypad().select = false,
                Key::Left => memory.joypad().left = false,
                Key::Right => memory.joypad().right = false,
                Key::Up => memory.joypad().up = false,
                Key::Down => memory.joypad().down = false,
                Key::Space => cpu.set_user_speed_multiplier(1),
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
                &mut texture_context,
                &texture,
            );
            cpu.reset_available_ccycles();
        });

        // Actions to do on update
        event.update(|_update_args| {
            while cpu.has_available_ccycles() {
                cpu.step();
                gpu.step(cpu.get_last_instruction_ccycles(), &mut canvas);
                audio_unit.step(cpu.get_last_instruction_ccycles());

                let check_vblank;
                let check_lcd_stat;
                let check_timer_overflow;
                let check_joystick;

                {
                    let mut memory = memory.borrow_mut();

                    check_vblank = memory.interrupt_enable().is_vblank()
                        && memory.interrupt_flag().is_vblank();

                    check_lcd_stat = memory.interrupt_enable().is_lcd_stat()
                        && memory.interrupt_flag().is_lcd_stat();

                    check_timer_overflow = memory.interrupt_enable().is_timer_overflow()
                        && memory.interrupt_flag().is_timer_overflow();

                    check_joystick = memory.interrupt_enable().is_p10_p13_transition()
                        && memory.interrupt_flag().is_p10_p13_transition();
                }

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
        });
    }
}
