use minifb::Key;

/// Currently configured for the following keys:
/// ```
/// index        keys
/// 1 2 3 C      1 2 3 4
/// 4 5 6 D  =>  Q W E R
/// 7 8 9 E  =>  A S D F
/// A 0 B F      Y X C V
/// ```
pub struct Keypad {
    pub keys: [bool; 16],
    pub waiting_for_release: Option<usize>,
}

impl Keypad {
    pub fn new() -> Self {
        Keypad {
            keys: [false; 16],
            waiting_for_release: None,
        }
    }

    fn clear(&mut self) {
        self.keys = [false; 16];
    }

    pub fn update_keypad(&mut self, keys: Option<Vec<Key>>) {
        self.clear();

        keys.map(|keys| {
            for key in keys {
                let index = match key {
                    Key::Key1 => 0x1,
                    Key::Key2 => 0x2,
                    Key::Key3 => 0x3,
                    Key::Key4 => 0xC,
                    Key::Q => 0x4,
                    Key::W => 0x5,
                    Key::E => 0x6,
                    Key::R => 0xD,
                    Key::A => 0x7,
                    Key::S => 0x8,
                    Key::D => 0x9,
                    Key::F => 0xE,
                    Key::Y => 0xA,
                    Key::X => 0x0,
                    Key::C => 0xB,
                    Key::V => 0xF,
                    _ => continue,
                };

                self.keys[index] = true;
            }
        });
    }
}

/// These are font sprites representing the hexadecimal numbers 0-F
/// Each font character is 4 pixels wide and 5 pixels tall
pub const FONT_SPRITES: [u8; 5 * 16] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,  // F
];