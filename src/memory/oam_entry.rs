pub struct OamEntry {
    position_y: u8,
    position_x: u8,
    tile_number: u8,
    flags: u8,
}

impl OamEntry {
    pub fn from_bytes(position_y: u8, position_x: u8, tile_number: u8, flags: u8) -> OamEntry {
        OamEntry {
            position_y,
            position_x,
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

    pub fn priority(&self) -> bool {
        self.flags & 0b1000 == 0b1000
    }

    pub fn tile_number(&self) -> u8 {
        self.tile_number
    }

    pub fn palette(&self) -> u8 {
        self.flags & 0x1
    }

    pub fn flip_y(&self) -> bool {
        self.flags & 0b100 == 0b100
    }

    pub fn flip_x(&self) -> bool {
        self.flags & 0b10 == 0b10
    }
}
