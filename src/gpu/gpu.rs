use crate::memory::memory::Memory;
use crate::gpu::color::Color;
use piston_window::*;
use crate::pause;

pub struct GPU {
    cycles_acumulated: u16,
}

impl GPU {
    pub fn new() -> GPU {
        return GPU {
            cycles_acumulated: 0,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8, memory: &mut Memory)
    {
        self.cycles_acumulated += last_instruction_cycles as u16;
        
        match memory.stat.mode {
            // H-blank mode
            0 => {
                if self.cycles_acumulated >= 204 {
                    self.cycles_acumulated = 0;
                    memory.ly += 1;

                    if memory.ly == 143 {
                        // Enter V-blank mode
                        memory.stat.mode = 1;
                        // TODO
                    } else {
                        // Enter Searching OAM-RAM mode
                        memory.stat.mode = 2;
                    }
                }
            }

            // V-blank mode
            1 => {
                if self.cycles_acumulated >= 456 {
                    self.cycles_acumulated = 0;
                    memory.ly += 1;

                    if memory.ly > 153 {
                        // Enter Searching OAM-RAM mode
                        memory.stat.mode = 2;
                        memory.ly = 0;
                    }
                }
            }

            // Searching OAM-RAM mode
            2 =>  {
                if self.cycles_acumulated >= 80 {
                    // Enter transferring data to LCD Driver mode
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 3;
                }
            }

            // Transferring data to LCD Driver mode
            3 =>  {
                if self.cycles_acumulated >= 172 {
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 0;

                    // TODO
                }
            }
            _ => panic!("Invalid GPU STAT mode")
        }
    }

    pub fn render(&mut self, window: & mut PistonWindow, event: &Event, window_size: [f64; 2], memory: &Memory) {
        const PIXEL_WIDTH: u8 = 160;
        const PIXEL_HEIGHT: u8 = 144;
        const TILE_SIZE_BYTES : u8 = 16;
        const BACKGROUND_MAP_TILE_SIZE_X: u16 = 32;
        const PIXELS_PER_TILE: u16 = 8;

        let tmp = window_size.get(0).unwrap();
        let tmp: f64 = *tmp / (PIXEL_WIDTH as f64);

        let pixel_size: (f64, f64) = (
            window_size.get(0).unwrap() / (PIXEL_WIDTH as f64),
            window_size.get(1).unwrap() / (PIXEL_HEIGHT as f64)
        );

        window.draw_2d(event, |context, graphics, _device| {
            let lcdc = &memory.lcdc;

            clear(Color::WHITE, graphics);

            if !lcdc.lcd_control_operation() {
                return;
            }

            let bg_tile_map_start_location : u16 = if lcdc.bg_tile_map_display_select() {0x9C00} else { 0x9800 };
            let bg_data_start_location : u16 = if lcdc.bg_and_window_tile_data_select() {0x8000} else { 0x8800 };
            let scx = memory.scx();
            let scy = memory.scy();

            //println!("{:X}", scy);
            let bgp = memory.bgp();

            let transform = context.transform;

            for screen_y in 0..(PIXEL_HEIGHT as u16) {
                for screen_x in 0..(PIXEL_WIDTH as u16) {
                    // Background
                    let bg_tile_map_location = bg_tile_map_start_location
                        + (screen_y / PIXELS_PER_TILE) * BACKGROUND_MAP_TILE_SIZE_X
                        + (screen_x / PIXELS_PER_TILE);

                    let bg_data_location = bg_data_start_location
                        + memory.read_8(bg_tile_map_location) as u16 * TILE_SIZE_BYTES as u16;

                    let tile_row = screen_y as u16 % 8;

                    let byte1 = memory.read_8(bg_data_location + tile_row * 2);
                    let byte2 = memory.read_8(bg_data_location + tile_row * 2 + 1);

                    let bit_pos = 7 - (screen_x % 8);

                    let pixel_bit1 = (byte1 >> bit_pos) & 0b1;
                    let pixel_bit0 = (byte2 >> bit_pos) & 0b1;

                    let pixel = ((pixel_bit1 << 1) | pixel_bit0) & 0b11;
                    let pixel = match pixel {
                        0b11 => bgp >> 6,
                        0b10 => bgp >> 4,
                        0b01 => bgp >> 2,
                        0b00 => bgp >> 0,
                        _ => panic!("Unrecognised color")
                    } & 0b11;

                    let color = match pixel {
                        0b00 => Color::WHITE,
                        0b01 => Color::DARK_GRAY,
                        0b10 => Color::GRAY,
                        0b11 => Color::BLACK,
                        _ => panic!("Unrecognised color")
                    };

                    let x_pixel = (screen_x as u16) + (7 - bit_pos as u16);
                    let y_pixel = (screen_y as u16) + (tile_row as u16);

                    let square = rectangle::rectangle_by_corners(
                        screen_x as f64 * pixel_size.0,
                        screen_y as f64 * pixel_size.1,
                        (screen_x + 1) as f64 * pixel_size.0,
                        (screen_y + 1) as f64 * pixel_size.1
                    );

                    rectangle(color, square, transform, graphics);
                }
            }
        });
    }
}