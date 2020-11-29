use crate::Byte;

#[derive(Default)]
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

    pub fn to_byte(&self) -> Byte {
        if !self.p15 && !self.p14 {
            println!("{:#010b}", 0xFF);
            return 0xFF;
        }

        let mut value = (!self.p15 as Byte) << 5;
        value |= (!self.p14 as Byte) << 4;

        if self.p15 {
            value |= (!(self.start) as Byte) << 3;
            value |= (!(self.select) as Byte) << 2;
            value |= (!(self.b) as Byte) << 1;
            value |= !(self.a) as Byte;
        } else if self.p14 {
            value |= (!(self.down) as Byte) << 3;
            value |= (!(self.up) as Byte) << 2;
            value |= (!(self.left) as Byte) << 1;
            value |= !(self.right) as Byte;
        }

        if value & 0x0F != 0x0F {
            println!("{:#010b}", value);
        }

        value
    }

    pub fn from_byte(&mut self, new_value: Byte) {
        self.p14 = new_value & 0b10000 != 0b10000;
        self.p15 = new_value & 0b100000 != 0b100000;
    }
}
