pub struct OamEntry {
    position_x: u8,
    position_y: u8,
    tile_number: u8,
    flags: u8,
}

impl OamEntry {
    pub fn from_bytes(position_x: u8, position_y: u8, tile_number: u8, flags: u8) -> OamEntry {
        OamEntry {
            position_x,
            position_y,
            tile_number,
            flags,
        }
    }
}

