use crate::Byte;

pub struct OamEntry {
    position_y: Byte,
    position_x: Byte,
    tile_number: Byte,
    flags: Byte,
}

impl OamEntry {
    pub fn with_bytes(
        position_y: Byte,
        position_x: Byte,
        tile_number: Byte,
        flags: Byte,
    ) -> OamEntry {
        OamEntry {
            position_y,
            position_x,
            tile_number,
            flags,
        }
    }

    pub fn x(&self) -> Byte {
        self.position_x
    }

    pub fn y(&self) -> Byte {
        self.position_y
    }

    pub fn priority(&self) -> bool {
        self.flags & 0b1000 == 0b1000
    }

    pub fn tile_number(&self) -> Byte {
        self.tile_number
    }

    pub fn palette(&self) -> Byte {
        self.flags & 0x1
    }

    pub fn flip_y(&self) -> bool {
        self.flags & 0b100 == 0b100
    }

    pub fn flip_x(&self) -> bool {
        self.flags & 0b10 == 0b10
    }
}
