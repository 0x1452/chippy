extern crate minifb;

use chippy::chip8::Chip8;
use chippy::display::Display;
use io::Read;
use std::io;
use std::time::Duration;
use std::{error, fs, path::Path};

use minifb::{Key, Scale, Window, WindowOptions};

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut chip8 = Chip8::new(&Path::new("roms/TETRIS"))?;

    let mut options = WindowOptions::default();
    options.scale = Scale::X8;

    let mut window = Window::new("TEST", 64, 32, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // 60hz
    window.limit_update_rate(Some(std::time::Duration::from_micros(16667)));
    // slow lol
    //window.limit_update_rate(Some(std::time::Duration::from_millis(500)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !chip8.done() { 
            chip8.run();
        }

        chip8.keypad.update_keypad(window.get_keys());

        if chip8.display.is_dirty {
            window
                .update_with_buffer(&chip8.display.screen, 64, 32)
                .unwrap();
            
            chip8.display.is_dirty = false;
        }
        //chip8.debug_print();

        //io::stdin().bytes().next();
    }

    Ok(())
}
