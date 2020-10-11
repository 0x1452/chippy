extern crate minifb;

use chippy::chip8::Chip8;
use std::{error, path::Path};
use std::time::Instant;

use minifb::{Key, Scale, Window, WindowOptions};

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut chip8 = Chip8::new(&Path::new("roms/BRIX"))?;

    let mut options = WindowOptions::default();
    options.scale = Scale::X8;

    let mut window = Window::new("TEST", 64, 32, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut now = Instant::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.keypad.update_keypad(window.get_keys());

        // TODO: most games seem to require different timing
        //       should be adjustable by the user and/or provide default values for known games
        if now.elapsed().as_micros() <= 1428*2 {
            continue;
        }

        if !chip8.done() { 
            chip8.run();
        }

        if chip8.display.is_dirty {
            window
                .update_with_buffer(&chip8.display.screen, 64, 32)
                .unwrap();

            chip8.display.is_dirty = false;
        }
        
        //chip8.debug_print();

        now = Instant::now();
    }

    Ok(())
}
