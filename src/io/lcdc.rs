use crate::Byte;

#[derive(Clone, Copy)]
#[readonly::make]
pub struct Lcdc {
    // 0 - Stop completely (no picture on screen)
    // 1 - operation
    pub lcd_control_operation: bool,

    // 0 - $9800-$9BFF
    // 1 - $9C00-$9DFF
    pub window_tile_map_display_select: bool,

    // 0 - off
    // 1 - on
    pub window_display: bool,

    // 0 - $8800-$97FF
    // 1 - $8000-$8FFF
    pub bg_and_window_tile_data_select: bool,

    // 0 - $9800-$9BFF
    // 1 - $9C00-$9DFF
    pub bg_tile_map_display_select: bool,

    // 0 - 8*8
    // 1 - 8*16
    pub obj_sprite_size: bool,

    // 0 - off
    // 1 - on
    pub obj_sprite_display: bool,

    // 0 - off
    // 1 - on
    pub bg_display: bool,
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

impl Default for Lcdc {
    fn default() -> Self {
        Lcdc::from(0x91)
    }
}
