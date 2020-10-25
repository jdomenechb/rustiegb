use crate::memory::memory::Memory;
use crate::gpu::color::Color;
use piston_window::*;

pub struct GPU {
    cycles_acumulated: u16,
}

impl GPU {
    pub fn new() -> GPU {
        return GPU {
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

    pub fn render(&mut self, window: & mut PistonWindow, event: &Event, window_size: [f64; 2]) {
        let pixel_size: (f64, f64) = (
            window_size.get(0).unwrap() / 160.0,
            window_size.get(1).unwrap() / 144.0
        );

        let square = rectangle::rectangle_by_corners(0.0, 0.0, pixel_size.0, pixel_size.1);
    
        window.draw_2d(event, |context, graphics, _device| {
            clear(Color::WHITE, graphics);

            let transform = context.transform;

            // Draw a box rotating around the middle of the screen.
            rectangle(Color::BLACK, square, transform, graphics);
        });
    }
}