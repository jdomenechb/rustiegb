use crate::memory::memory::Memory;
use crate::gpu::color::Color;
use piston_window::*;
use gfx_device_gl::{Factory, Resources, CommandBuffer};
use ::image::{RgbaImage, Rgba};
use crate::memory::oam_entry::OamEntry;

type DisplayPixel = [u8;4];

pub struct GPU {
    cycles_acumulated: u16,
    sprites_to_be_drawn: Vec<OamEntry>,
}

impl GPU {
    pub const PIXEL_WIDTH: u8 = 160;
    pub const PIXEL_HEIGHT: u8 = 144;
    const TILE_SIZE_BYTES: u8 = 16;
    const BACKGROUND_MAP_TILE_SIZE_X: u16 = 32;
    const BACKGROUND_MAP_TILE_SIZE_Y: u16 = 32;
    const PIXELS_PER_TILE: u16 = 8;

    pub fn new() -> GPU {
        return GPU {
            cycles_acumulated: 0,
            sprites_to_be_drawn: Vec::with_capacity(10),
        };
    }

    pub fn step(&mut self, last_instruction_cycles: u8, memory: &mut Memory, canvas: &mut RgbaImage)
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
                        memory.interrupt_flag().set_vblank(true);
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
            2 => {
                if self.cycles_acumulated >= 80 {
                    // Enter transferring data to LCD Driver mode
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 3;

                    self.sprites_to_be_drawn.clear();

                    let lcdc = &memory.lcdc;

                    if !lcdc.obj_sprite_display() {
                        return;
                    }

                    let ly = memory.ly;

                    for oam_entry in memory.oam_ram() {
                        if
                            oam_entry.x() != 0
                            && ly + 16 >= oam_entry.y()
                            && ly + 16 < oam_entry.y() + 8 // FIXME: Replace by actual sprite height
                        {
                            self.sprites_to_be_drawn.push(oam_entry);
                        }
                    }
                }
            }

            // Transferring data to LCD Driver mode
            3 => {
                if self.cycles_acumulated >= 172 {
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 0;

                    // Draw pixel line
                    let lcdc = &memory.lcdc;

                    if !lcdc.lcd_control_operation() {
                        return;
                    }

                    let bg_tile_map_start_location: u16 = if lcdc.bg_tile_map_display_select() { 0x9C00 } else { 0x9800 };
                    let bg_data_start_location: u16 = if lcdc.bg_and_window_tile_data_select() { 0x8000 } else { 0x8800 };

                    let scx = memory.scx();
                    let scy = memory.scy();
                    let bgp = memory.bgp();
                    let screen_y = memory.ly as u16;

                    for screen_x in 0..(GPU::PIXEL_WIDTH as u16) {
                        let mut pixel_to_write: Option<DisplayPixel> = None;

                        // Sprites with high priority
                        if lcdc.obj_sprite_display() {
                            pixel_to_write = self.draw_sprites(memory, true, screen_x, screen_y);
                        }

                        if lcdc.bg_display() {
                            // Background
                            let screen_y_with_offset = scy as u16 + screen_y;
                            let screen_x_with_offset = scx as u16 + screen_x;

                            let bg_tile_map_location = bg_tile_map_start_location
                                + (((screen_y_with_offset / GPU::PIXELS_PER_TILE) * GPU::BACKGROUND_MAP_TILE_SIZE_X) % (GPU::BACKGROUND_MAP_TILE_SIZE_X * GPU::BACKGROUND_MAP_TILE_SIZE_Y))
                                + (screen_x_with_offset / GPU::PIXELS_PER_TILE);

                            let bg_data_location = bg_data_start_location
                                + memory.read_8(bg_tile_map_location) as u16 * GPU::TILE_SIZE_BYTES as u16;

                            let tile_row = screen_y_with_offset as u16 % 8;
                            let tile_x = screen_x_with_offset % 8;

                            let pixel = self.read_pixel_from_tile(memory,bg_data_location, tile_row, tile_x);
                            let pixel_color = match pixel {
                                0b11 => bgp >> 6,
                                0b10 => bgp >> 4,
                                0b01 => bgp >> 2,
                                0b00 => bgp >> 0,
                                _ => panic!("Unrecognised color")
                            } & 0b11;

                            let color = match pixel_color {
                                0b00 => Color::white(),
                                0b01 => Color::dark_grey(),
                                0b10 => Color::light_grey(),
                                0b11 => Color::black(),
                                _ => panic!("Unrecognised color")
                            };

                            if pixel != 0x0 || pixel_to_write.is_none() {
                                pixel_to_write = Some(color.to_rgba());
                            }
                        }

                        // Sprites with high priority
                        if lcdc.obj_sprite_display() {
                            let tmp = self.draw_sprites(memory,false, screen_x, screen_y);

                            if tmp.is_some() {
                                pixel_to_write = tmp;
                            }
                        }

                        if pixel_to_write.is_some() {
                            canvas.put_pixel(screen_x as u32, screen_y as u32, Rgba(pixel_to_write.unwrap()));
                        }
                    }
                }
            }
            _ => panic!("Invalid GPU STAT mode")
        }
    }

    fn read_pixel_from_tile(&self, memory: &Memory, tile_address: u16, row: u16, x: u16) -> u8 {
        let byte1 = memory.read_8(tile_address + row * 2);
        let byte2 = memory.read_8(tile_address + row * 2 + 1);

        let bit_pos = 7 - x;

        let pixel_bit1 = (byte1 >> bit_pos) & 0b1;
        let pixel_bit0 = (byte2 >> bit_pos) & 0b1;

        ((pixel_bit1 << 1) | pixel_bit0) & 0b11
    }

    fn draw_sprites(&self, memory: &Memory, priority: bool, screen_x: u16, screen_y: u16) -> Option<DisplayPixel> {
        const SPRITE_TILES_ADDR_START: u16 = 0x8000;

        let mut pixel_to_write = None;
        let mut last_drawn: Option<&OamEntry> = None;

        for sprite in &self.sprites_to_be_drawn {
            if priority != sprite.priority() {
                continue;
            }

            if last_drawn.is_some() && last_drawn.unwrap().x() < sprite.x() {
                continue;
            }

            last_drawn = Some(sprite);

            // TODO: 8x16 mode
            let current_pixel_x :i16 = screen_x as i16 + GPU::PIXELS_PER_TILE as i16  - sprite.x() as i16;

            if current_pixel_x < 0 || current_pixel_x >= 8 {
                continue;
            }
            
            let current_pixel_y :i16 = screen_y as i16 + (GPU::PIXELS_PER_TILE * 2) as i16 - sprite.y() as i16;

            if current_pixel_y < 0 || current_pixel_y >= 8 {
                continue;
            }

            let sprite_addr = SPRITE_TILES_ADDR_START
                + sprite.tile_number() as u16 * GPU::TILE_SIZE_BYTES as u16;

            let pixel = self.read_pixel_from_tile(memory, sprite_addr, current_pixel_x as u16, current_pixel_y as u16);

            if pixel == 0 {
                continue;
            }

            let palette = memory.read_8(if sprite.palette() == 0 { 0xFF48 } else {0xFF49});

            let pixel_color = match pixel {
                0b11 => palette >> 6,
                0b10 => palette >> 4,
                0b01 => palette >> 2,
                0b00 => palette >> 0,
                _ => panic!("Unrecognised color")
            } & 0b11;

            let color = match pixel_color {
                0b00 => Color::white(),
                0b01 => Color::dark_grey(),
                0b10 => Color::light_grey(),
                0b11 => Color::black(),
                _ => panic!("Unrecognised color")
            };

            pixel_to_write = Some(color.to_rgba())
        }

        pixel_to_write
    }

    pub fn render(
        &mut self,
        window: &mut PistonWindow,
        event: &Event,
        window_size: [f64; 2],
        memory: &Memory,
        texture_context: &mut TextureContext<Factory, Resources, CommandBuffer>,
        texture: &Texture<Resources>,
    ) {
        let pixel_size: (f64, f64) = (
            window_size.get(0).unwrap() / (GPU::PIXEL_WIDTH as f64),
            window_size.get(1).unwrap() / (GPU::PIXEL_HEIGHT as f64)
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