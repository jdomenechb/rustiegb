use crate::Byte;

#[readonly::make]
pub struct OamEntry {
    pub y: Byte,
    pub x: Byte,
    pub tile_number: Byte,
    flags: Byte,
}

impl OamEntry {
    pub fn with_bytes(y: Byte, x: Byte, tile_number: Byte, flags: Byte) -> OamEntry {
        OamEntry {
            y,
            x,
            tile_number,
            flags,
        }
    }

    pub fn priority(&self) -> bool {
        self.flags & 0b10000000 == 0b10000000
    }

    pub fn palette(&self) -> bool {
        self.flags & 0b10000 == 0b10000
    }

    pub fn flip_y(&self) -> bool {
        self.flags & 0b1000000 == 0b1000000
    }

    pub fn flip_x(&self) -> bool {
        self.flags & 0b100000 == 0b100000
    }
}
