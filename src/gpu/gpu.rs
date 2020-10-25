use crate::memory::memory::Memory;
use crate::gpu::color::Color;
use piston_window::*;
use crate::pause;

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

    pub fn render(&mut self, window: & mut PistonWindow, event: &Event, window_size: [f64; 2], memory: &Memory) {
        let pixel_size: (f64, f64) = (
            window_size.get(0).unwrap() / 160.0,
            window_size.get(1).unwrap() / 144.0
        );

        window.draw_2d(event, |context, graphics, _device| {
            clear(Color::WHITE, graphics);

            let transform = context.transform;

            for byte_location in 0..32 {
                let mem_location = 0x104 + byte_location;
                let byte = memory.read_8(mem_location);

                for bit_pos in (0..8).rev() {
                    let bit = (byte >> bit_pos) & 0b1;
                    let color = match bit {
                        1 => Color::BLACK,
                        0 => Color::WHITE,
                        _ => panic!("Unrecognised color")
                    };

                    let x = (pixel_size.0 * 4.0 * (byte_location / 2) as f64) + (pixel_size.0 * (3.0 - ((bit_pos as f64) % 4.0)));
                    let y = (pixel_size.1 * 2.0 * (byte_location as f64 % 2.0)) + pixel_size.1 * (1 - (bit_pos / 4)) as f64;

                    //println!("{}, {}, {}", x, y, bit);

                    let square = rectangle::rectangle_by_corners(x, y, x + pixel_size.0, y + pixel_size.1);

                    rectangle(color, square, transform, graphics);
                }

                //pause();
            }
        });
    }
}