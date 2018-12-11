use piston::input::*;
use opengl_graphics::{ GlGraphics };
use graphics::*;
use super::super::memory::memory::Memory;
use super::color::Color;

pub struct GPU {
    gl: GlGraphics,
    cycles_acumulated: u16,
}

impl GPU {
    pub fn new(gl: GlGraphics) -> GPU {
        return GPU {
            gl: gl,
            cycles_acumulated: 0,
        }
    }

    pub fn step(&mut self, last_instruction_cycles: u8, memory: &mut Memory)
    {
        self.cycles_acumulated += last_instruction_cycles as u16;
        
        match memory.stat.mode {
            // H-blank mode
            0 => {
                if self.cycles_acumulated >= 204 {
                    self.cycles_acumulated = 0;
                    memory.ly += 1;

                    if memory.ly == 143 {
                        // Enter V-blank mode
                        memory.stat.mode = 1;
                        // TODO
                    } else {
                        // Enter Searching OAM-RAM mode
                        memory.stat.mode = 2;
                    }
                }
            }

            // V-blank mode
            1 => {
                if self.cycles_acumulated >= 456 {
                    self.cycles_acumulated = 0;
                    memory.ly += 1;

                    if memory.ly > 153 {
                        // Enter Searching OAM-RAM mode
                        memory.stat.mode = 2;
                        memory.ly = 0;
                    }
                }
            }

            // Searching OAM-RAM mode
            2 =>  {
                if self.cycles_acumulated >= 80 {
                    // Enter transferring data to LCD Driver mode
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 3;
                }
            }

            // Transferring data to LCD Driver mode
            3 =>  {
                if self.cycles_acumulated >= 172 {
                    self.cycles_acumulated = 0;
                    memory.stat.mode = 0;

                    // TODO
                }
            }
            _ => panic!("Invalid GPU STAT mode")
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
        // TODO
    }
}