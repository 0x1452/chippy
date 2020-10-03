extern crate minifb;

use chippy::chip8::Chip8;
use chippy::display::Display;
use std::io;
use std::time::Duration;
use std::{error, fs, path::Path};

use minifb::{Key, Scale, Window, WindowOptions};

fn main() -> Result<(), Box<dyn error::Error>> {
    //check_roms(&Path::new("roms"))?;
    let mut chip8 = Chip8::new();
    chip8.load_rom(&Path::new("roms/test_opcode.ch8"))?;

    let mut options = WindowOptions::default();
    options.scale = Scale::X8;

    let mut window = Window::new("TEST", 64, 32, options).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.run();

        window
            .update_with_buffer(&chip8.display.screen, 64, 32)
            .unwrap();
    }

    Ok(())
}
