use crate::gpu::color::Color;
use crate::math::word_to_two_bytes;
use crate::memory::memory::Memory;
use crate::memory::oam_entry::OamEntry;
use crate::memory::stat::STATMode;
use crate::{Byte, Word};
use ::image::{ImageBuffer, Rgba, RgbaImage};
use gfx_device_gl::{CommandBuffer, Factory, Resources};
use piston_window::*;
use std::cell::RefCell;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::rc::Rc;

type DisplayPixel = [Byte; 4];

pub struct GPU {
    cycles_accumulated: u16,

    sprites_to_be_drawn_with_priority: Vec<OamEntry>,
    sprites_to_be_drawn_without_priority: Vec<OamEntry>,

    memory: Rc<RefCell<Memory>>,

    tile_row_cache: RefCell<HashMap<(Word, u16), (Byte, Byte)>>,

    last_window_rendered_position_x: u16,
    last_window_rendered_position_y: u16,
}

impl GPU {
    pub const PIXEL_WIDTH: u8 = 160;
    pub const PIXEL_HEIGHT: u8 = 144;

    const TILE_SIZE_BYTES: u8 = 16;
    const BACKGROUND_MAP_TILE_SIZE_X: u16 = 32;
    const BACKGROUND_MAP_TILE_SIZE_Y: u16 = 32;
    const PIXELS_PER_TILE: u16 = 8;

    pub fn new(memory: Rc<RefCell<Memory>>) -> GPU {
        return GPU {
            cycles_accumulated: 0,
            sprites_to_be_drawn_with_priority: Vec::with_capacity(10),
            sprites_to_be_drawn_without_priority: Vec::with_capacity(10),
            memory,
            tile_row_cache: RefCell::new(HashMap::new()),
            last_window_rendered_position_x: 0,
            last_window_rendered_position_y: 0,
        };
    }

    pub fn step(&mut self, last_instruction_cycles: u8, canvas: &mut RgbaImage) {
        let mode;

        {
            let memory = self.memory.borrow();
            mode = memory.stat.mode();
        }

        self.cycles_accumulated += last_instruction_cycles as u16;

        match mode {
            // H-blank mode
            STATMode::HBlank => self.hblank(),

            // V-blank mode
            STATMode::VBlank => self.vblank(),

            // Searching OAM-RAM mode
            STATMode::SearchOamRam => self.search_oam_ram(),

            // Transferring data to LCD Driver mode
            STATMode::LCDTransfer => self.lcd_transfer(canvas),
        }
    }

    fn hblank(&mut self) {
        if self.cycles_accumulated >= 204 {
            self.cycles_accumulated = 0;

            {
                let mut memory = self.memory.borrow_mut();
                memory.ly_increment();

                if memory.ly.has_reached_end_of_screen() {
                    memory.set_stat_mode(STATMode::VBlank);
                } else {
                    memory.set_stat_mode(STATMode::SearchOamRam);
                }
            }
        }
    }

    fn vblank(&mut self) {
        if self.cycles_accumulated >= 456 {
            self.cycles_accumulated = 0;

            self.last_window_rendered_position_x = 0;
            self.last_window_rendered_position_y = 0;

            {
                let mut memory = self.memory.borrow_mut();
                memory.ly_increment();

                if memory.ly.has_reached_end_of_vblank() {
                    // Enter Searching OAM-RAM mode
                    memory.set_stat_mode(STATMode::SearchOamRam);
                    memory.ly_reset();
                    self.tile_row_cache.borrow_mut().clear();
                }
            }
        }
    }

    fn search_oam_ram(&mut self) {
        if self.cycles_accumulated < 80 {
            return;
        }

        // Enter transferring data to LCD Driver mode
        self.cycles_accumulated = 0;

        let mut memory = self.memory.borrow_mut();
        memory.set_stat_mode(STATMode::LCDTransfer);

        self.sprites_to_be_drawn_with_priority.clear();
        self.sprites_to_be_drawn_without_priority.clear();

        let lcdc = &memory.lcdc;

        if !lcdc.obj_sprite_display() {
            return;
        }

        let ly: u8 = memory.ly.clone().into();
        let sprite_size = if lcdc.obj_sprite_size() { 16 } else { 8 };

        for oam_entry in memory.oam_ram() {
            if oam_entry.x() != 0
                && ly + 16 >= oam_entry.y()
                && ly + 16 < oam_entry.y() + sprite_size
            {
                if oam_entry.priority() {
                    self.sprites_to_be_drawn_with_priority.push(oam_entry);
                } else {
                    self.sprites_to_be_drawn_without_priority.push(oam_entry);
                }
            }
        }

        self.sprites_to_be_drawn_with_priority
            .sort_by(|a, b| a.x().cmp(&b.x()));

        self.sprites_to_be_drawn_without_priority
            .sort_by(|a, b| a.x().cmp(&b.x()));
    }

    fn lcd_transfer(&mut self, canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
        if self.cycles_accumulated < 172 {
            return;
        }

        self.cycles_accumulated = 0;

        {
            let mut memory = self.memory.borrow_mut();
            memory.set_stat_mode(STATMode::HBlank);
        }

        let memory = self.memory.borrow();

        // Draw pixel line
        let lcdc = &memory.lcdc;

        if !lcdc.lcd_control_operation() {
            return;
        }

        let bg_tile_map_start_location = if lcdc.bg_tile_map_display_select() {
            0x9C00
        } else {
            0x9800
        };

        let window_tile_map_start_location = if lcdc.window_tile_map_display_select() {
            0x9C00
        } else {
            0x9800
        };

        let scx = memory.scx();
        let scy = memory.scy();
        let bgp = memory.bgp();

        let screen_y = Byte::from(memory.ly.clone()) as u16;
        let screen_y_with_offset = scy as u16 + screen_y;

        let mut previous_bg_tile_map_location = 0u16;
        let mut tile_bytes = (0, 0);

        let sprite_palette0 = memory.read_byte(0xFF48);
        let sprite_palette1 = memory.read_byte(0xFF49);

        let sprite_size = if memory.lcdc.obj_sprite_size() {
            16i16
        } else {
            8i16
        };

        let mut any_window_rendered = false;
        let mut screen_row: [Option<DisplayPixel>; GPU::PIXEL_WIDTH as usize] =
            [None; GPU::PIXEL_WIDTH as usize];

        if lcdc.obj_sprite_display() {
            self.draw_sprites_in_row(
                false,
                screen_y,
                sprite_palette0,
                sprite_palette1,
                sprite_size,
                &mut screen_row,
            );
        }

        for screen_x in 0..(GPU::PIXEL_WIDTH as u16) {
            let pixel_to_write = screen_row.get(screen_x as usize).unwrap();

            if pixel_to_write.is_some() {
                canvas.put_pixel(
                    screen_x as u32,
                    screen_y as u32,
                    Rgba(pixel_to_write.unwrap()),
                );

                continue;
            }

            let mut pixel_to_write: Option<DisplayPixel> = None;
            let screen_x_with_offset = ((screen_x as u8).wrapping_add(scx)) as u16;
            let tile_x;

            // Sprites with low priority
            if lcdc.obj_sprite_display() {
                pixel_to_write = self.draw_sprites(
                    true,
                    screen_x,
                    screen_y,
                    sprite_palette0,
                    sprite_palette1,
                    sprite_size,
                );
            }

            if lcdc.bg_display() {
                let bg_tile_map_location;
                let tile_row;

                // Window
                if lcdc.window_display()
                    && memory.wy <= screen_y as Byte
                    && memory.wx <= (screen_x + 7) as Byte
                {
                    bg_tile_map_location = window_tile_map_start_location
                        + (((self.last_window_rendered_position_y / GPU::PIXELS_PER_TILE)
                            * GPU::BACKGROUND_MAP_TILE_SIZE_X)
                            % (GPU::BACKGROUND_MAP_TILE_SIZE_X * GPU::BACKGROUND_MAP_TILE_SIZE_Y))
                        + (self.last_window_rendered_position_x / GPU::PIXELS_PER_TILE);

                    tile_x = self.last_window_rendered_position_x % 8;
                    tile_row = self.last_window_rendered_position_y % 8;

                    self.last_window_rendered_position_x += 1;
                    any_window_rendered = true;
                } else {
                    // Background
                    bg_tile_map_location = bg_tile_map_start_location
                        + (((screen_y_with_offset / GPU::PIXELS_PER_TILE)
                            * GPU::BACKGROUND_MAP_TILE_SIZE_X)
                            % (GPU::BACKGROUND_MAP_TILE_SIZE_X * GPU::BACKGROUND_MAP_TILE_SIZE_Y))
                        + (screen_x_with_offset / GPU::PIXELS_PER_TILE);

                    tile_x = screen_x_with_offset % 8;
                    tile_row = screen_y_with_offset as u16 % 8;
                }

                if previous_bg_tile_map_location != bg_tile_map_location {
                    let bg_data_location = match lcdc.bg_and_window_tile_data_select() {
                        true => {
                            0x8000
                                + memory.read_byte(bg_tile_map_location) as Word
                                    * GPU::TILE_SIZE_BYTES as Word
                        }
                        false => {
                            let rel_address = memory.read_byte(bg_tile_map_location);

                            (if rel_address >= 0b10000000 {
                                0x8800
                            } else {
                                0x9000
                            }) + (rel_address & 0b01111111) as Word * GPU::TILE_SIZE_BYTES as Word
                        }
                    };

                    tile_bytes = self.read_tile_row(bg_data_location, tile_row);

                    previous_bg_tile_map_location = bg_tile_map_location;
                }

                let pixel = self.read_pixel_from_tile(tile_x, tile_bytes);

                if pixel != 0x0 || pixel_to_write.is_none() {
                    let pixel_color = match pixel {
                        0b11 => bgp >> 6,
                        0b10 => bgp >> 4,
                        0b01 => bgp >> 2,
                        0b00 => bgp >> 0,
                        _ => panic!("Unrecognised color"),
                    } & 0b11;

                    let color = match pixel_color {
                        0b00 => Color::white(),
                        0b01 => Color::light_grey(),
                        0b10 => Color::dark_grey(),
                        0b11 => Color::black(),
                        _ => panic!("Unrecognised color"),
                    };

                    pixel_to_write = Some(color.to_rgba());
                }
            }

            if pixel_to_write.is_some() {
                canvas.put_pixel(
                    screen_x as u32,
                    screen_y as u32,
                    Rgba(pixel_to_write.unwrap()),
                );
            }
        }

        self.last_window_rendered_position_x = 0;

        if any_window_rendered {
            self.last_window_rendered_position_y += 1;
        }
    }

    fn read_pixel_from_tile(&self, bit_offset: u16, (byte1, byte2): (Byte, Byte)) -> Byte {
        let bit_pos = 7 - bit_offset;

        let pixel_bit1 = (byte1 >> bit_pos) & 0b1;
        let pixel_bit0 = (byte2 >> bit_pos) & 0b1;

        ((pixel_bit1 << 1) | pixel_bit0) & 0b11
    }

    fn read_tile_row(&self, tile_address: Word, row: u16) -> (Byte, Byte) {
        //let key = (tile_address, row);

        // let mut cache = self.tile_row_cache.borrow_mut();
        //
        // match cache.get(&key) {
        //     Some(result) => return *result,
        //     _ => {}
        // }

        let memory = self.memory.borrow();

        let word = memory.read_word(tile_address + row * 2);
        let bytes = word_to_two_bytes(word);

        // cache.insert((tile_address, row), bytes);

        bytes
    }

    fn draw_sprites(
        &self,
        priority: bool,
        screen_x: u16,
        screen_y: u16,
        palette0: Byte,
        palette1: Byte,
        sprite_size: i16,
    ) -> Option<DisplayPixel> {
        const SPRITE_TILES_ADDR_START: u16 = 0x8000;

        let sprites_to_be_drawn = match priority {
            true => &self.sprites_to_be_drawn_with_priority,
            false => &self.sprites_to_be_drawn_without_priority,
        };

        let mut last_drawn: Option<&OamEntry> = None;

        for sprite in sprites_to_be_drawn {
            let current_pixel_x: i16 =
                screen_x as i16 + GPU::PIXELS_PER_TILE as i16 - sprite.x() as i16;

            if current_pixel_x < 0 || current_pixel_x >= 8 {
                continue;
            }

            let current_pixel_y: i16 =
                screen_y as i16 + (GPU::PIXELS_PER_TILE * 2) as i16 - sprite.y() as i16;

            let sprite_addr =
                SPRITE_TILES_ADDR_START + sprite.tile_number() as u16 * GPU::TILE_SIZE_BYTES as u16;

            let row = if sprite.flip_y() {
                sprite_size - 1 - current_pixel_y
            } else {
                current_pixel_y
            } as Word;

            let tile_row = self.read_tile_row(sprite_addr, row);

            let pixel = self.read_pixel_from_tile(
                if sprite.flip_x() {
                    7 - current_pixel_x
                } else {
                    current_pixel_x
                } as Word,
                tile_row,
            );

            if pixel == 0 {
                continue;
            }

            last_drawn = Some(sprite);

            let palette = if !sprite.palette() {
                palette0
            } else {
                palette1
            };

            let pixel_color = match pixel {
                0b11 => palette >> 6,
                0b10 => palette >> 4,
                0b01 => palette >> 2,
                _ => panic!("Unrecognised color"),
            } & 0b11;

            let color = match pixel_color {
                0b00 => Color::white(),
                0b01 => Color::light_grey(),
                0b10 => Color::dark_grey(),
                0b11 => Color::black(),
                _ => panic!("Unrecognised color"),
            };

            return Some(color.to_rgba());
        }

        None
    }

    fn draw_sprites_in_row(
        &self,
        priority: bool,
        screen_y: u16,
        palette0: Byte,
        palette1: Byte,
        sprite_size: i16,
        screen_row: &mut [Option<DisplayPixel>],
    ) {
        const SPRITE_TILES_ADDR_START: u16 = 0x8000;

        let sprites_to_be_drawn = match priority {
            true => &self.sprites_to_be_drawn_with_priority,
            false => &self.sprites_to_be_drawn_without_priority,
        };

        let mut screen_x = -1;

        for sprite in sprites_to_be_drawn {
            screen_x = max(
                screen_x + 1,
                sprite.x() as i16 - GPU::PIXELS_PER_TILE as i16,
            );

            let current_pixel_y: i16 =
                screen_y as i16 + (GPU::PIXELS_PER_TILE * 2) as i16 - sprite.y() as i16;

            let sprite_addr =
                SPRITE_TILES_ADDR_START + sprite.tile_number() as u16 * GPU::TILE_SIZE_BYTES as u16;

            let row = if sprite.flip_y() {
                sprite_size - 1 - current_pixel_y
            } else {
                current_pixel_y
            } as Word;

            let tile_row = self.read_tile_row(sprite_addr, row);

            let limit = min(sprite.x() as i16, GPU::PIXEL_WIDTH as i16);

            for current_screen_x in screen_x..limit {
                let current_pixel_x: i16 =
                    current_screen_x as i16 + GPU::PIXELS_PER_TILE as i16 - sprite.x() as i16;

                if current_pixel_x < 0 || current_pixel_x >= 8 {
                    continue;
                }

                let pixel = self.read_pixel_from_tile(
                    if sprite.flip_x() {
                        7 - current_pixel_x
                    } else {
                        current_pixel_x
                    } as Word,
                    tile_row,
                );

                if pixel == 0 {
                    continue;
                }

                last_drawn = Some(sprite);

                let palette = if !sprite.palette() {
                    palette0
                } else {
                    palette1
                };

                let pixel_color = match pixel {
                    0b11 => palette >> 6,
                    0b10 => palette >> 4,
                    0b01 => palette >> 2,
                    _ => panic!("Unrecognised color"),
                } & 0b11;

                let color = match pixel_color {
                    0b00 => Color::white(),
                    0b01 => Color::light_grey(),
                    0b10 => Color::dark_grey(),
                    0b11 => Color::black(),
                    _ => panic!("Unrecognised color"),
                };

                screen_row[current_screen_x as usize] = Some(color.to_rgba());
                screen_x = current_screen_x;
            }
        }
    }

    pub fn render(
        &mut self,
        window: &mut PistonWindow,
        event: &Event,
        window_size: [f64; 2],
        texture_context: &mut TextureContext<Factory, Resources, CommandBuffer>,
        texture: &Texture<Resources>,
    ) {
        let memory = self.memory.borrow();

        let pixel_size: (f64, f64) = (
            window_size.get(0).unwrap() / (GPU::PIXEL_WIDTH as f64),
            window_size.get(1).unwrap() / (GPU::PIXEL_HEIGHT as f64),
        );

        window.draw_2d(event, |context, graphics, device| {
            texture_context.encoder.flush(device);

            clear(Color::white().to_f_rgba(), graphics);

            let lcdc = &memory.lcdc;

            if !lcdc.lcd_control_operation() {
                return;
            }

            let transform = context.transform;
            let transform = transform.scale(pixel_size.0, pixel_size.1);

            image(texture, transform, graphics);
        });
    }
}
