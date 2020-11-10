pub struct OamEntry {
    position_x: u8,
    position_y: u8,
    tile_number: u8,
    flags: u8,
}

impl OamEntry {
    pub fn new() -> OamEntry {
        OamEntry {
            position_x: 0,
            position_y: 0,
            tile_number: 0,
            flags: 0,
        }
    }
}

