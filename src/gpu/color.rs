use crate::Byte;

pub struct Color {
    r: Byte,
    g: Byte,
    b: Byte,
}

impl Color {
    pub fn new(r: Byte, g: Byte, b: Byte) -> Self {
        Self { r, g, b }
    }

    pub fn from_pixel(pixel: Byte, palette: Byte) -> Self {
        let pixel_color = match pixel {
            0b11 => palette >> 6,
            0b10 => palette >> 4,
            0b01 => palette >> 2,
            0b00 => palette,
            _ => panic!("Unrecognised color"),
        } & 0b11;

        Self::from(pixel_color)
    }

    pub fn black() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn dark_grey() -> Self {
        Self::new(85, 85, 85)
    }

    pub fn light_grey() -> Self {
        Self::new(170, 170, 170)
    }

    pub fn white() -> Self {
        Self::new(255, 255, 255)
    }

    pub fn to_f_rgba(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            1.0,
        ]
    }

    pub fn to_rgba(&self) -> [Byte; 4] {
        [self.r, self.g, self.b, 255]
    }
}

impl From<Byte> for Color {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Self::white(),
            0b01 => Self::light_grey(),
            0b10 => Self::dark_grey(),
            0b11 => Self::black(),
            _ => panic!("Unrecognised color"),
        }
    }
}
