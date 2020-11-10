pub struct LCDC {
    // 0 - Stop completely (no picture on screen)
    // 1 - operation
    lcd_control_operation: bool,

    // 0 - $9800-$9BFF
    // 1 - $9C00-$9DFF
    window_tile_map_display_select: bool,

    // 0 - off
    // 1 - on
    window_display: bool,

    // 0 - $8800-$97FF              
    // 1 - $8000-$8FFF
    bg_and_window_tile_data_select: bool,

    // 0 - $9800-$9BFF
    // 1 - $9C00-$9DFF
    bg_tile_map_display_select: bool,

    // 0 - 8*8
    // 1 - 8*16
    obj_sprite_size: bool,

    // 0 - off
    // 1 - on
    obj_sprite_display: bool,

    // 0 - off
    // 1 - on
    bg_display: bool,
}

impl LCDC {
    pub fn new() -> LCDC {
        return LCDC {
            lcd_control_operation: false,
            window_tile_map_display_select: true,
            window_display: false,
            bg_and_window_tile_data_select: true,
            bg_tile_map_display_select: true,
            obj_sprite_size: false,
            obj_sprite_display: true,
            bg_display: true,
        }
    }

    pub fn from_u8(&mut self, value: u8) {
        self.lcd_control_operation = value & 0b10000000 == 0b10000000;
        self.window_tile_map_display_select = value & 0b01000000 == 0b01000000;
        self.window_display = value & 0b00100000 == 0b00100000;
        self.bg_and_window_tile_data_select = value & 0b10000 == 0b10000;
        self.bg_tile_map_display_select = value & 0b1000 == 0b1000;
        self.obj_sprite_size = value & 0b100 == 0b100;
        self.obj_sprite_display = value & 0b10 == 0b10;
        self.bg_display = value & 0b1 == 0b1;
    }

    pub fn to_u8(&self) -> u8 {
        let value =
            ((self.lcd_control_operation as u8) << 7)
            | ((self.window_tile_map_display_select as u8) << 6)
            | ((self.window_display as u8) << 5)
            | ((self.bg_and_window_tile_data_select as u8) << 4)
            | ((self.bg_tile_map_display_select as u8) << 3)
            | ((self.obj_sprite_size as u8) << 2)
            | ((self.obj_sprite_display as u8) << 1)
            | (self.bg_display as u8);

        value
    }

    pub fn lcd_control_operation(&self) -> bool {
        return self.lcd_control_operation;
    }

    pub fn window_tile_map_display_select(&self) -> bool {
        return self.window_tile_map_display_select;
    }

    pub fn window_display(&self) -> bool {
        return self.window_display;
    }

    pub fn bg_and_window_tile_data_select(&self) -> bool {
        return self.bg_and_window_tile_data_select;
    }

    pub fn bg_tile_map_display_select(&self) -> bool {
        return self.bg_tile_map_display_select;
    }

    pub fn obj_sprite_size(&self) -> bool {
        return self.obj_sprite_size;
    }

    pub fn obj_sprite_display(&self) -> bool {
        return self.obj_sprite_display;
    }

    pub fn bg_display(&self) -> bool {
        return self.bg_display;
    }

}