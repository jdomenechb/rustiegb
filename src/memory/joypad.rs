pub struct Joypad {
    // P14 - P10
    pub right: bool,
    // P14 - P11
    pub left: bool,
    // P14 - P12
    pub up: bool,
    // P14 - P13
    pub down: bool,

    // P15 - P10
    pub a: bool,
    // P15 - P11
    pub b: bool,
    // P15 - P12
    pub select: bool,
    // P15 - P13
    pub start: bool,

    p14: bool,
    p15: bool,
}

impl Joypad {
    pub fn new() -> Joypad {
        Joypad {
            right: false,
            left: false,
            up: false,
            down: false,

            a: false,
            b: false,
            select: false,
            start: false,

            p14: false,
            p15: false,
        }
    }

    pub fn to_u8(&self) -> u8 {
        let mut value = (!self.p15 as u8) << 5;
        value |= (!self.p14 as u8) << 4;

        if self.p14 {
            value |= (!self.down as u8) << 3;
            value |= (!self.up as u8) << 2;
            value |= (!self.left as u8) << 1;
            value |= !self.right as u8;
        } else if self.p15 {
            value |= (!self.start as u8) << 3;
            value |= (!self.select as u8) << 2;
            value |= (!self.b as u8) << 1;
            value |= !self.a as u8;
        } else {
            value |= 0b1111;
        }

        value
    }

    pub fn from_u8(&mut self, new_value: u8) {
        self.p14 = new_value & 0b10000 == 0b10000;
        self.p15 = new_value & 0b100000 == 0b100000;
    }
}