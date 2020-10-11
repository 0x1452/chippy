use std::path::Path;
use std::error;
use rand::random;
use crate::display::Display;
use crate::display;
use crate::keypad::{Keypad, FONT_SPRITES};

const FONT_START: usize = 0x50;
const PROGRAM_START: usize = 0x200;

pub struct Chip8 {
    v: [u8; 16],
    i: u16,
    sp: u16,
    pc: u16,
    dt: u8,     // delay timer
    st: u8,     // sound timer
    stack: [u16; 16],
    memory: [u8; 4096],
    pub keypad: Keypad,
    pub display: Display,
}

impl Chip8 {
    fn init() -> Self {
        Chip8 {
            v: [0; 16],
            i: 0,
            sp: 0,
            pc: 0x200,
            dt: 0,
            st: 0,
            stack: [0; 16],
            memory: [0; 4096],
            keypad: Keypad::new(),
            display: Display::new(),
        }
    }

    pub fn new<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn error::Error>> {
        let mut chip8 = Chip8::init();

        chip8.load_rom(path)?;
        chip8.load_font();

        Ok(chip8)
    }

    fn load_rom<P: AsRef<Path>>(&mut self, path: &P) -> Result<(), Box<dyn error::Error>> {
        let data = std::fs::read(path)?;

        if data.len() >= self.memory.len() - PROGRAM_START {
            return Err("ROM size has to be below 3583 bytes.")?;
        }

        for i in 0..data.len() {
            self.memory[PROGRAM_START + i] = data[i];
        };

        Ok(())
    }

    fn load_font(&mut self) {
        for i in 0..FONT_SPRITES.len() {
            self.memory[FONT_START + i] = FONT_SPRITES[i];
        }
    }

    pub fn done(&self) -> bool {
        return self.memory[self.pc as usize] == 0 
            && self.memory[self.pc as usize + 1] == 0;
    }

    pub fn debug_print(&self) {
        for i in 0..self.v.len() {
            print!("V{:X}={:X} ", i, self.v[i]);
        }
        println!();
        println!("I={:X} SP={:X} PC={:X} DT={} ST={}", self.i, self.sp, self.pc, self.dt, self.st);
        print!("Stack=[");
        for item in self.stack.iter() {
            print!("{:X}, ", item)
        }
        println!("]");
        println!("Keypad = {:?}", self.keypad.keys);
    }

    fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    /// Run one fetch-decode-execute cycle
    pub fn run(&mut self) {
        self.update_timers();
        // Fetch
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);

        let op1 = (opcode >> 12 & 0xF) as usize;
        let x = (opcode >> 8 & 0xF) as usize;
        let y = (opcode >> 4 & 0xF) as usize;
        let op2 = (opcode & 0xF) as usize;

        let addr = ((x << 8) | (y << 4) | (op2)) as u16;
        let byte = ((y << 4) | (op2)) as u8;
        
        //println!("{}", self.disassemble_op(opcode));

        // Decode and Execute
        match (op1, x, y, op2) {
            // CLS
            //  Clear the screen
            (0x0, 0x0, 0xE, 0x0) => self.display.cls(),
            // RET
            //  Return from subroutine
            //  Set the program counter to the address at the top of the stack,
            //  then subtract 1 from the stack pointer
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;

                return;
            },
            // this would execute a machine language subroutine written at address NNN
            (0, _, _, _) => {},
            // JMP addr
            //  Jump to address
            (0x1, _, _, _) => {
                self.pc = addr;
                return;
            },
            // CALL addr
            //  Call subroutine at address
            (0x2, _, _, _) => {
                assert!(self.sp < 15, "CALL -> called subroutine with full stack");

                self.sp += 1;
                self.stack[self.sp as usize] = self.pc + 2;

                self.pc = addr;
                return;
            },
            // SE Vx, byte
            //  Skip next instruction if Vx = byte
            (0x3, _, _, _) => {
                if self.v[x] == byte {
                    self.pc += 2;
                }
            },
            // SNE Vx, byte
            //  Skip next instruction if Vx != byte
            (0x4, _, _, _) => {
                if self.v[x] != byte {
                    self.pc += 2;
                }
            },
            // SE Vx, Vy
            //  Skip next instruction if Vx = Vy
            (0x5, _, _, 0x0) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            },
            // LD Vx, byte
            //  Vx = byte
            (0x6, _, _, _) => self.v[x] = byte,
            // ADD Vx, byte
            //  Vx += byte
            (0x7, _, _, _) => self.v[x] = self.v[x].overflowing_add(byte).0,
            // LD Vx, Vy
            //  Vx = Vy
            (0x8, _, _, 0x0) => self.v[x] = self.v[y],
            // OR Vx, Vy
            (0x8, _, _, 0x1) => self.v[x] |= self.v[y],
            // AND Vx, Vy
            (0x8, _, _, 0x2) => self.v[x] &= self.v[y],
            // XOR Vx, Vy
            (0x8, _, _, 0x3) => self.v[x] ^= self.v[y],
            // ADD Vx, Vy
            //  set VF = carry
            (0x8, _, _, 0x4) => {
                let (res, carry) = self.v[x].overflowing_add(self.v[y]);

                self.v[x] = res;
                self.v[0xF] = carry as u8;
            },
            // SUB Vx, Vy
            //  Vx = Vx - Vy
            //  VF = Vx > Vy ? 1 : 0
            (0x8, _, _, 0x5) => {
                self.v[0xF] = (self.v[x] > self.v[y]) as u8;
                self.v[x] = self.v[x].overflowing_sub(self.v[y]).0;
            },
            // SHR Vx {, Vy}
            //  set VF = LSB of Vx
            //  Vx = Vx >> 1
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            },
            // SUBN Vx, Vy
            //  Vx = Vy - Vx
            //  VF = Vy > Vx ? 1 : 0
            (0x8, _, _, 0x7) => {
                self.v[x] = self.v[y].overflowing_sub(self.v[x]).0;
                self.v[0xF] = (self.v[y] > self.v[x]) as u8;
            },
            // SHL Vx {, Vy}
            //  VF = MSB of Vx
            //  Vx = Vx << 1;
            (0x8, _, _, 0xE) => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            },
            // SNE Vx, Vy
            //  Skip next instruction if Vx != Vy
            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            },
            // LD I, addr
            (0xA, _, _, _) => self.i = addr,
            // JP V0, addr
            //  Jump to V0+addr
            (0xB, _, _, _) => self.pc = self.v[0] as u16 + addr,
            // RND Vx, byte
            //  Generate random number between 0 and 255
            //  Vx = random number & `byte`
            (0xC, _, _, _) => {
                let rand: u8 = random();

                self.v[x] &= rand;
            },
            // DRW Vx, Vy, nibble
            //  Display n-byte sprite starting at memory location I at (Vx, Vy)
            //  VF = collision
            //  Sprites are XORed onto the existing screen -> if it causes pixels to be erased set VF
            //      No idea if sprites are supposed to wrap
            (0xD, _, _, _) => {
                let (x, y) = (self.v[x], self.v[y]);
                let mut collision = false;

                // for `op2` bytes in memory
                for row in 0..op2 {
                    let coord_y = (y + row as u8) as usize;
                    if coord_y >= display::HEIGHT {
                        break;
                    }
                    // get sprite byte from memory
                    let byte = self.memory[self.i as usize + row];

                    // for bit in byte
                    for bit_offset in 0..8 {
                        let coord_x = ((x + bit_offset) as u8) as usize;
                        if coord_x >= display::WIDTH {
                            break;
                        }

                        let bit = (byte >> (7 - bit_offset)) & 1;

                        if bit == 1 {
                            if self.display.toggle(coord_x, coord_y) {
                                collision = true;
                            }
                        }
                    }
                }

                self.v[0xF] = collision as u8;
            },
            // SKP Vx
            //  Skip next instruction if key with the value of Vx is pressed
            (0xE, _, 0x9, 0xE) => {
                if self.keypad.keys[self.v[x] as usize] {
                    self.pc += 2;
                }
            },
            // SKNP Vx
            //  Skip next instruction if key with the value of Vx is not pressed
            (0xE, _, 0xA, 0x1) => {
                if !self.keypad.keys[self.v[x] as usize] {
                    self.pc += 2;
                }
            },
            // LD Vx, DT
            //  Vx = delay timer value
            (0xF, _, 0x0, 0x7) => {
                self.v[x] = self.dt;
            },
            // LD Vx, K
            //  Wait for a key press, then store the value of the key in Vx 
            //  Execution stops until a key is pressed.
            //  https://retrocomputing.stackexchange.com/a/361 describes this instruction in great detail
            (0xF, _, 0x0, 0xA) => {
                match self.keypad.waiting_for_release {
                    Some(key) => {
                        self.keypad.waiting_for_release = None;
                        self.v[x] = key as u8;
                    },
                    None => {
                        // Get the first key that is pressed rn
                        for key in 0..self.keypad.keys.len() {
                            if self.keypad.keys[key] {
                                self.keypad.waiting_for_release = Some(key);
                            }
                        } 

                        return;
                    }
                }
            },
            // LD DT, Vx
            //  DT = Vx
            (0xF, _, 0x1, 0x5) => self.dt = self.v[x],
            // LD ST, Vx
            //  ST = Vx
            (0xF, _, 0x1, 0x8) => self.st = self.v[x],
            // ADD I, Vx
            //  I += Vx
            (0xF, _, 0x1, 0xE) => self.i += self.v[x] as u16,
            // LD F, Vx
            //  I = location of sprite Vx
            (0xF, _, 0x2, 0x9) => {
                self.i = self.v[x] as u16 * 5;
            },
            // LD B, Vx
            //  Stores the BCD (binary-coded decimal) representation of Vx into I, I+1, and I+2
            //  *(I+0) = (Vx // 100)
            //  *(I+1) = (Vx  % 100) // 10
            //  *(I+2) = (Vx  %  10) 
            (0xF, _, 0x3, 0x3) => {
                self.memory[self.i as usize] = self.v[x] / 100; 
                self.memory[self.i as usize + 1] = (self.v[x] % 100) / 10;
                self.memory[self.i as usize + 2] = self.v[x] % 10; 
            },
            // LD [I], Vx
            //  Store registers V0 - Vx in memory region starting at location I
            //  *(I+0) = V0
            //  *(I+1) = V1
            //  ...
            //  *(I+x) = Vx
            (0xF, _, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.i as usize + i] = self.v[i];
                }
            },
            // LD Vx, [I]
            // Read registers V0 - Vx from memory region starting at location I
            // V0 = *(I+0)
            // V1 = *(I+1)
            // ...
            // Vx = *(I+x)
            (0xF, _, 0x6, 0x5) => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i as usize + i];
                }
            },
            _ => unimplemented!("OPCODE not implemented: {:04X}", opcode),
        };

        self.pc += 2;
    }
    
    #[allow(dead_code)]
    fn disassemble_op(&self, opcode: u16) -> String {
        let op1 = opcode >> 12 & 0xF;
        let x = opcode >> 8 & 0xF;
        let y = opcode >> 4 & 0xF;
        let op2 = opcode & 0xF;

        let addr = (x << 8) | (y << 4) | (op2);
        let byte = (y << 4) | (op2);

        let decompiled = match (op1, x, y, op2) {
            (0x0, 0x0, 0xE, 0x0) => String::from("CLS"),
            (0x0, 0x0, 0xE, 0xE) => String::from("RET"),
            // this would execute a machine language subroutine written at address NNN
            (0, _, _, _) => format!("SYS  {:X} \t // Would call machine code", addr),
            (0x1, _, _, _) => format!("JMP  {:X}", addr),
            (0x2, _, _, _) => format!("CALL {:X}", addr),
            (0x3, _, _, _) => format!("SE   V{:X}, {:X}", x, byte),
            (0x4, _, _, _) => format!("SNE  V{:X}, {:X}", x, byte),
            (0x5, _, _, 0x0) => format!("SE   V{:X}, V{:X}", x, y),
            (0x6, _, _, _) => format!("LD   V{:X}, {:X}", x, byte),
            (0x7, _, _, _) => format!("ADD  V{:X}, {:X}", x, byte),
            (0x8, _, _, 0x0) => format!("LD   V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x1) => format!("OR   V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x2) => format!("AND  V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x3) => format!("XOR  V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x4) => format!("ADD  V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x5) => format!("SUB  V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x6) => format!("SHR  V{:X}, V{:X}", x, y),
            (0x8, _, _, 0x7) => format!("SUBN V{:X}, V{:X}", x, y),
            (0x8, _, _, 0xE) => format!("SHL  V{:X}, V{:X}", x, y),
            (0x9, _, _, 0x0) => format!("SNE  V{:X}, V{:X}", x, y),
            (0xA, _, _, _) => format!("LD   I, {:X}", addr),
            (0xB, _, _, _) => format!("JP   V0, {:X}", addr),
            (0xC, _, _, _) => format!("RND  V{:X}, {:X}", x, byte),
            (0xD, _, _, _) => format!("DRW  V{:X}, V{:X}, {:X}", x, y, op2),
            (0xE, _, 0x9, 0xE) => format!("SKP  V{:X}", x),
            (0xE, _, 0xA, 0x1) => format!("SKNP V{:X}", x),
            (0xF, _, 0x0, 0x7) => format!("LD   V{:X}, DT", x),
            (0xF, _, 0x0, 0xA) => format!("LD   V{:X}, K", x),
            (0xF, _, 0x1, 0x5) => format!("LD   DT, V{:X}", x),
            (0xF, _, 0x1, 0x8) => format!("LD   ST, V{:X}", x),
            (0xF, _, 0x1, 0xE) => format!("ADD  I, V{:X}", x),
            (0xF, _, 0x2, 0x9) => format!("LD   F, V{:X}", x),
            (0xF, _, 0x3, 0x3) => format!("LD   B, V{:X}", x),
            (0xF, _, 0x5, 0x5) => format!("LD   [I], V{:X}", x),
            (0xF, _, 0x6, 0x5) => format!("LD   V{:X}, [I]", x),
            //_ => unimplemented!("OPCODE not implemented: {:04X}", opcode),
            _ => format!("OPCODE not implemented: {:04X}", opcode),
        };

        return format!("[{:04X}] {:04X} | {}", self.pc, opcode, decompiled);
    }
}
