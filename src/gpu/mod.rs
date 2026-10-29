use std::cmp::{max, min};
use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use parking_lot::RwLock;

use crate::gpu::color::Color;
use crate::io::registers::IORegisters;
use crate::io::stat::STATMode;
use crate::memory::Memory;
use crate::memory::memory_sector::{ReadMemory, ReadWordMemory};
use crate::memory::oam_entry::OamEntry;
use crate::utils::math::word_to_two_bytes;
use crate::{Byte, Word};

pub mod color;

type DisplayPixel = [Byte; 4];

pub struct Gpu {
    cycles_accumulated: u16,

    sprites_to_be_drawn_with_priority: Vec<OamEntry>,
    sprites_to_be_drawn_without_priority: Vec<OamEntry>,

    memory: Arc<RwLock<Memory>>,
    io_registers: Arc<RwLock<IORegisters>>,
    canvas: Arc<RwLock<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
}

impl Gpu {
    pub const PIXEL_WIDTH: u8 = 160;
    pub const PIXEL_HEIGHT: u8 = 144;

    const TILE_SIZE_BYTES: u8 = 16;
    const BACKGROUND_MAP_TILE_SIZE_X: u16 = 32;
    const BACKGROUND_MAP_TILE_SIZE_Y: u16 = 32;
    const PIXELS_PER_TILE: u16 = 8;

    pub fn new(
        memory: Arc<RwLock<Memory>>,
        io_registers: Arc<RwLock<IORegisters>>,
        canvas: Arc<RwLock<ImageBuffer<Rgba<u8>, Vec<u8>>>>,
    ) -> Gpu {
        Gpu {
            cycles_accumulated: 0,
            sprites_to_be_drawn_with_priority: Vec::with_capacity(10),
            sprites_to_be_drawn_without_priority: Vec::with_capacity(10),
            memory,
            io_registers,
            canvas,
        }
    }

    pub fn single_step(&mut self) {
        let mode;
        let lcdc;

        {
            let io_registers = self.io_registers.read();
            mode = io_registers.stat.mode();
            lcdc = io_registers.lcdc;
        }

        if !lcdc.lcd_control_operation {
            {
                self.io_registers.write().ly_reset_wo_interrupt();
            }
            self.cycles_accumulated = 0;

            return;
        }

        self.cycles_accumulated += 1;

        match mode {
            // H-blank mode
            STATMode::HBlank => self.hblank(),

            // V-blank mode
            STATMode::VBlank => self.vblank(),

            // Searching OAM-RAM mode
            STATMode::SearchOamRam => self.search_oam_ram(),

            // Transferring data to LCD Driver mode
            STATMode::LCDTransfer => self.lcd_transfer(),
        }
    }

    fn hblank(&mut self) {
        if self.cycles_accumulated >= 204 {
            self.cycles_accumulated = 0;

            {
                let mut io_registers = self.io_registers.write();
                io_registers.ly_increment();

                if io_registers.ly.has_reached_end_of_screen() {
                    io_registers.set_stat_mode(STATMode::VBlank);
                } else {
                    io_registers.set_stat_mode(STATMode::SearchOamRam);
                }
            }
        }
    }

    fn vblank(&mut self) {
        if self.cycles_accumulated >= 456 {
            self.cycles_accumulated = 0;

            {
                let mut io_registers = self.io_registers.write();
                io_registers.ly_increment();

                if io_registers.ly.has_reached_end_of_vblank() {
                    // Enter Searching OAM-RAM mode
                    io_registers.set_stat_mode(STATMode::SearchOamRam);
                    io_registers.ly_reset();
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

        self.sprites_to_be_drawn_with_priority.clear();
        self.sprites_to_be_drawn_without_priority.clear();

        let obj_sprite_display;
        let obj_sprite_size;
        let ly;

        {
            let mut io_registers = self.io_registers.write();
            io_registers.set_stat_mode(STATMode::LCDTransfer);

            obj_sprite_display = io_registers.lcdc.obj_sprite_display;
            obj_sprite_size = io_registers.lcdc.obj_sprite_size;
            ly = io_registers.ly.value;
        }

        if !obj_sprite_display {
            return;
        }

        let sprite_size = if obj_sprite_size { 16 } else { 8 };

        let oam_ram = { self.memory.read().oam_ram.clone() };

        for oam_entry in oam_ram {
            if oam_entry.x != 0 && ly + 16 >= oam_entry.y && ly + 16 < oam_entry.y + sprite_size {
                if oam_entry.priority() {
                    self.sprites_to_be_drawn_with_priority.push(oam_entry);
                } else {
                    self.sprites_to_be_drawn_without_priority.push(oam_entry);
                }
            }
        }

        self.sprites_to_be_drawn_with_priority.sort_by_key(|a| a.x);

        self.sprites_to_be_drawn_without_priority
            .sort_by_key(|a| a.x);
    }

    fn lcd_transfer(&mut self) {
        if self.cycles_accumulated < 172 {
            return;
        }

        self.cycles_accumulated = 0;

        {
            self.io_registers.write().set_stat_mode(STATMode::HBlank);
        }

        let lcdc;
        let scx;
        let scy;
        let bgp;
        let screen_y;
        let sprite_palette0;
        let sprite_palette1;
        let sprite_size;

        {
            let io_registers = self.io_registers.read();

            lcdc = io_registers.lcdc;
            scx = io_registers.scx;
            scy = io_registers.scy;
            bgp = io_registers.bgp;

            screen_y = io_registers.ly.value as u16;
            sprite_palette0 = io_registers.obp1;
            sprite_palette1 = io_registers.obp2;

            sprite_size = if io_registers.lcdc.obj_sprite_size {
                16i16
            } else {
                8i16
            };
        }

        if !lcdc.lcd_control_operation {
            return;
        }

        let bg_tile_map_start_location = if lcdc.bg_tile_map_display_select {
            0x9C00
        } else {
            0x9800
        };

        let window_tile_map_start_location = if lcdc.window_tile_map_display_select {
            0x9C00
        } else {
            0x9800
        };

        let screen_y_with_offset = scy as u16 + screen_y;

        let mut previous_bg_tile_map_location = 0u16;
        let mut tile_bytes = (0, 0);

        let mut screen_row_no_priority: [Option<DisplayPixel>; Gpu::PIXEL_WIDTH as usize] =
            [None; Gpu::PIXEL_WIDTH as usize];
        let mut screen_row_with_priority: [Option<DisplayPixel>; Gpu::PIXEL_WIDTH as usize] =
            [None; Gpu::PIXEL_WIDTH as usize];

        if lcdc.obj_sprite_display {
            self.draw_sprites_in_row(
                false,
                screen_y,
                sprite_palette0,
                sprite_palette1,
                sprite_size,
                &mut screen_row_no_priority,
            );

            self.draw_sprites_in_row(
                true,
                screen_y,
                sprite_palette0,
                sprite_palette1,
                sprite_size,
                &mut screen_row_with_priority,
            );
        }

        let mut canvas = self.canvas.write();

        for screen_x in 0..(Gpu::PIXEL_WIDTH as u16) {
            let mut pixel_to_write = *screen_row_no_priority.get(screen_x as usize).unwrap();

            if let Some(ptw) = pixel_to_write {
                {
                    canvas.put_pixel(screen_x as u32, screen_y as u32, Rgba(ptw));
                }
                continue;
            }

            pixel_to_write = *screen_row_with_priority.get(screen_x as usize).unwrap();

            if lcdc.bg_display {
                let screen_x_with_offset = ((screen_x as u8).wrapping_add(scx)) as u16;

                let wy;
                let wx;

                {
                    let io_registers = self.io_registers.read();
                    wy = io_registers.wy;
                    wx = io_registers.wx;
                }

                // Window
                let (bg_tile_map_location, tile_x, tile_row) = if lcdc.window_display
                    && wy <= screen_y as Byte
                    && wx <= (screen_x + 7) as Byte
                {
                    let last_window_rendered_position_x: u16 = screen_x + 7 - wx as u16;
                    let last_window_rendered_position_y = screen_y - wy as u16;

                    let bg_tile_map_location = window_tile_map_start_location
                        + (((last_window_rendered_position_y / Gpu::PIXELS_PER_TILE)
                            * Gpu::BACKGROUND_MAP_TILE_SIZE_X)
                            % (Gpu::BACKGROUND_MAP_TILE_SIZE_X * Gpu::BACKGROUND_MAP_TILE_SIZE_Y))
                        + (last_window_rendered_position_x / Gpu::PIXELS_PER_TILE);

                    (
                        bg_tile_map_location,
                        last_window_rendered_position_x % 8,
                        last_window_rendered_position_y % 8,
                    )
                } else {
                    // Background
                    let bg_tile_map_location = bg_tile_map_start_location
                        + (((screen_y_with_offset / Gpu::PIXELS_PER_TILE)
                            * Gpu::BACKGROUND_MAP_TILE_SIZE_X)
                            % (Gpu::BACKGROUND_MAP_TILE_SIZE_X * Gpu::BACKGROUND_MAP_TILE_SIZE_Y))
                        + (screen_x_with_offset / Gpu::PIXELS_PER_TILE);

                    (
                        bg_tile_map_location,
                        screen_x_with_offset % 8,
                        screen_y_with_offset % 8,
                    )
                };

                if previous_bg_tile_map_location != bg_tile_map_location {
                    let bg_tile_map = { self.memory.read().read_byte(bg_tile_map_location) };

                    let bg_data_location = match lcdc.bg_and_window_tile_data_select {
                        true => 0x8000 + bg_tile_map as Word * Gpu::TILE_SIZE_BYTES as Word,
                        false => {
                            (if bg_tile_map >= 0b10000000 {
                                0x8800
                            } else {
                                0x9000
                            }) + (bg_tile_map & 0b01111111) as Word * Gpu::TILE_SIZE_BYTES as Word
                        }
                    };

                    tile_bytes = self.read_tile_row(bg_data_location, tile_row);

                    previous_bg_tile_map_location = bg_tile_map_location;
                }

                let pixel = self.read_pixel_from_tile(tile_x, tile_bytes);

                if pixel != 0x0 || pixel_to_write.is_none() {
                    let color = Color::from_pixel(pixel, bgp);

                    pixel_to_write = Some(color.to_rgba());
                }
            }

            if let Some(ptw) = pixel_to_write {
                {
                    canvas.put_pixel(screen_x as u32, screen_y as u32, Rgba(ptw));
                }
            }
        }
    }

    fn read_pixel_from_tile(&self, bit_offset: u16, (byte1, byte2): (Byte, Byte)) -> Byte {
        let bit_pos = 7 - bit_offset;

        let pixel_bit1 = (byte1 >> bit_pos) & 0b1;
        let pixel_bit0 = (byte2 >> bit_pos) & 0b1;

        ((pixel_bit1 << 1) | pixel_bit0) & 0b11
    }

    fn read_tile_row(&self, tile_address: Word, row: u16) -> (Byte, Byte) {
        let word = { self.memory.read().read_word(tile_address + row * 2) };

        word_to_two_bytes(word)
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
            screen_x = max(screen_x + 1, sprite.x as i16 - Gpu::PIXELS_PER_TILE as i16);

            let current_pixel_y: i16 =
                screen_y as i16 + (Gpu::PIXELS_PER_TILE * 2) as i16 - sprite.y as i16;

            let sprite_addr =
                SPRITE_TILES_ADDR_START + sprite.tile_number as u16 * Gpu::TILE_SIZE_BYTES as u16;

            let row = if sprite.flip_y() {
                sprite_size - 1 - current_pixel_y
            } else {
                current_pixel_y
            } as Word;

            let tile_row = self.read_tile_row(sprite_addr, row);

            let limit = min(sprite.x as i16, Gpu::PIXEL_WIDTH as i16);
            let mut sprite_end = screen_x;

            let palette = if !sprite.palette() {
                palette0
            } else {
                palette1
            };

            for current_screen_x in screen_x..limit {
                let current_pixel_x: i16 =
                    current_screen_x + Gpu::PIXELS_PER_TILE as i16 - sprite.x as i16;

                if !(0..8).contains(&current_pixel_x) {
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

                let color = Color::from_pixel(pixel, palette);

                screen_row[current_screen_x as usize] = Some(color.to_rgba());
                sprite_end = current_screen_x;
            }

            screen_x = sprite_end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_fixture(lcd_on: bool) -> (Gpu, Arc<RwLock<IORegisters>>) {
        let memory = Arc::new(RwLock::new(Memory::default()));
        let io_registers = memory.read().io_registers.clone();
        let canvas = Arc::new(RwLock::new(ImageBuffer::new(
            Gpu::PIXEL_WIDTH as u32,
            Gpu::PIXEL_HEIGHT as u32,
        )));

        {
            let mut io_registers = io_registers.write();
            io_registers.lcdc.update(if lcd_on { 0x80 } else { 0x00 });
            io_registers.stat.set_mode(STATMode::SearchOamRam);
        }

        (Gpu::new(memory, io_registers.clone(), canvas), io_registers)
    }

    fn step(gpu: &mut Gpu, cycles: u32) {
        for _ in 0..cycles {
            gpu.single_step();
        }
    }

    fn mode(io_registers: &Arc<RwLock<IORegisters>>) -> STATMode {
        io_registers.read().stat.mode()
    }

    fn ly(io_registers: &Arc<RwLock<IORegisters>>) -> Byte {
        io_registers.read().ly.value
    }

    #[test]
    fn it_does_nothing_and_resets_ly_when_lcd_is_off() {
        let (mut fixture, io_registers) = create_fixture(false);
        io_registers.write().ly_increment();
        fixture.cycles_accumulated = 50;

        step(&mut fixture, 1000);

        assert_eq!(ly(&io_registers), 0);
        assert_eq!(fixture.cycles_accumulated, 0);
        assert!(matches!(mode(&io_registers), STATMode::SearchOamRam));
    }

    #[test]
    fn it_goes_through_the_modes_of_a_scanline() {
        let (mut fixture, io_registers) = create_fixture(true);

        step(&mut fixture, 79);
        assert!(matches!(mode(&io_registers), STATMode::SearchOamRam));
        step(&mut fixture, 1);
        assert!(matches!(mode(&io_registers), STATMode::LCDTransfer));

        step(&mut fixture, 171);
        assert!(matches!(mode(&io_registers), STATMode::LCDTransfer));
        step(&mut fixture, 1);
        assert!(matches!(mode(&io_registers), STATMode::HBlank));

        step(&mut fixture, 203);
        assert!(matches!(mode(&io_registers), STATMode::HBlank));
        assert_eq!(ly(&io_registers), 0);
        step(&mut fixture, 1);
        assert!(matches!(mode(&io_registers), STATMode::SearchOamRam));
        assert_eq!(ly(&io_registers), 1);
    }

    #[test]
    fn it_takes_456_cycles_per_scanline() {
        let (mut fixture, io_registers) = create_fixture(true);

        step(&mut fixture, 456 * 10);

        assert_eq!(ly(&io_registers), 10);
    }

    #[test]
    fn it_enters_vblank_and_requests_the_interrupt_at_the_end_of_the_screen() {
        let (mut fixture, io_registers) = create_fixture(true);

        step(&mut fixture, 456 * 143);

        assert_eq!(ly(&io_registers), 143);
        assert!(matches!(mode(&io_registers), STATMode::VBlank));
        assert!(io_registers.read().interrupt_flag.vblank);
    }

    #[test]
    fn it_does_not_request_vblank_before_the_end_of_the_screen() {
        let (mut fixture, io_registers) = create_fixture(true);

        step(&mut fixture, 456 * 142);

        assert!(!io_registers.read().interrupt_flag.vblank);
    }

    #[test]
    fn it_leaves_vblank_back_to_the_first_line() {
        let (mut fixture, io_registers) = create_fixture(true);

        // 144 visible lines + 10 vblank lines
        step(&mut fixture, 456 * 154);

        assert_eq!(ly(&io_registers), 0);
        assert!(matches!(mode(&io_registers), STATMode::SearchOamRam));
    }

    #[test]
    fn it_draws_the_scanline_into_the_shared_canvas() {
        let (mut fixture, io_registers) = create_fixture(true);
        io_registers.write().lcdc.update(0x91);
        io_registers.write().bgp = 0xFF;

        step(&mut fixture, 80 + 172);

        let canvas = fixture.canvas.read();
        let darkest = *canvas.get_pixel(0, 0);
        assert_ne!(darkest, Rgba([0, 0, 0, 0]));
    }
}
