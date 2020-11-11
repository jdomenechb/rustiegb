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

    pub fn x(&self) -> u8 {
        self.position_x
    }

    pub fn y(&self) -> u8 {
        self.position_y
    }

    pub fn tile_number(&self) -> u8 {
        self.tile_number
    }

    pub fn flags(&self) -> u8 {
        self.flags
    }
}

