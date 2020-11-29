use crate::Byte;

pub struct Color {
    r: Byte,
    g: Byte,
    b: Byte,
}

impl Color {
    pub fn new(r: Byte, g: Byte, b: Byte) -> Color {
        Color { r, g, b }
    }

    pub fn black() -> Color {
        Color::new(0, 0, 0)
    }

    pub fn dark_grey() -> Color {
        Color::new(85, 85, 85)
    }

    pub fn light_grey() -> Color {
        Color::new(170, 170, 170)
    }

    pub fn white() -> Color {
        Color::new(255, 255, 255)
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
