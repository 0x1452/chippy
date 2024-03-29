pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

const BLACK: u32 = 0;
const WHITE: u32 = 0x00FFFFFF;

pub struct Display {
    pub screen: [u32; WIDTH * HEIGHT],
    pub is_dirty: bool
}

impl Display {
    pub fn new() -> Display {
        Display {
            screen: [BLACK; WIDTH * HEIGHT],
            is_dirty: false,
        }
    }

    pub fn cls(&mut self) {
        self.screen = [BLACK; WIDTH * HEIGHT];
    }

    pub fn toggle(&mut self, x: usize, y: usize) -> bool {
        let index = x + WIDTH * y;
        let mut collision = false;

        self.screen[index] = match self.screen[index] {
            BLACK => WHITE,
            WHITE => {
                collision = true;
                BLACK
            },
            _ => panic!(),
        };

        self.is_dirty = true;

        collision
    }
}