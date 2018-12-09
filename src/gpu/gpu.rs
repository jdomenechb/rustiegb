use piston::input::*;
use opengl_graphics::{ GlGraphics };
use graphics::*;
use super::color::Color;

pub struct GPU {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl GPU {
    pub fn new(gl: GlGraphics) -> GPU {
        return GPU {
            gl: gl
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let pixel_size: (f64, f64) = (args.width / 160.0, args.height / 144.0);

        let square = rectangle::rectangle_by_corners(0.0, 0.0, pixel_size.0, pixel_size.1);
    
        self.gl.draw(args.viewport(), |c, gl| {
            clear(Color::WHITE, gl);

            let transform = c.transform;

            // Draw a box rotating around the middle of the screen.
            rectangle(Color::BLACK, square, transform, gl);
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
    }
}