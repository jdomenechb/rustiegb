use crate::Byte;

#[derive(Default, Clone, Copy)]
pub struct Lcdc {
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

impl Lcdc {
    pub fn new() -> Lcdc {
        Lcdc {
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

    pub fn to_byte(&self) -> Byte {
        ((self.lcd_control_operation as Byte) << 7)
            | ((self.window_tile_map_display_select as Byte) << 6)
            | ((self.window_display as Byte) << 5)
            | ((self.bg_and_window_tile_data_select as Byte) << 4)
            | ((self.bg_tile_map_display_select as Byte) << 3)
            | ((self.obj_sprite_size as Byte) << 2)
            | ((self.obj_sprite_display as Byte) << 1)
            | (self.bg_display as Byte)
    }

    pub fn lcd_control_operation(&self) -> bool {
        self.lcd_control_operation
    }

    pub fn window_tile_map_display_select(&self) -> bool {
        self.window_tile_map_display_select
    }

    pub fn window_display(&self) -> bool {
        self.window_display
    }

    pub fn bg_and_window_tile_data_select(&self) -> bool {
        self.bg_and_window_tile_data_select
    }

    pub fn bg_tile_map_display_select(&self) -> bool {
        self.bg_tile_map_display_select
    }

    pub fn obj_sprite_size(&self) -> bool {
        self.obj_sprite_size
    }

    pub fn obj_sprite_display(&self) -> bool {
        self.obj_sprite_display
    }

    pub fn bg_display(&self) -> bool {
        self.bg_display
    }
}

impl From<Byte> for Lcdc {
    fn from(value: Byte) -> Self {
        Self {
            lcd_control_operation: value & 0b10000000 == 0b10000000,
            window_tile_map_display_select: value & 0b01000000 == 0b01000000,
            window_display: value & 0b00100000 == 0b00100000,
            bg_and_window_tile_data_select: value & 0b10000 == 0b10000,
            bg_tile_map_display_select: value & 0b1000 == 0b1000,
            obj_sprite_size: value & 0b100 == 0b100,
            obj_sprite_display: value & 0b10 == 0b10,
            bg_display: value & 0b1 == 0b1,
        }
    }
}

impl From<&Lcdc> for Byte {
    fn from(original: &Lcdc) -> Self {
        ((original.lcd_control_operation as Byte) << 7)
            | ((original.window_tile_map_display_select as Byte) << 6)
            | ((original.window_display as Byte) << 5)
            | ((original.bg_and_window_tile_data_select as Byte) << 4)
            | ((original.bg_tile_map_display_select as Byte) << 3)
            | ((original.obj_sprite_size as Byte) << 2)
            | ((original.obj_sprite_display as Byte) << 1)
            | (original.bg_display as Byte)
    }
}
