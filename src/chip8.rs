use std::{fs::File, io, path::Path};

use std::error;
use rand::random;
use crate::display::Display;
use crate::display;

const PROGRAM_START: usize = 0x200;

pub struct Chip8 {
    v: [u8; 16],
    i: u16,
    sp: u16,
    pc: u16,
    stack: [u16; 16],
    memory: [u8; 4096],
    pub display: Display,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            v: [0; 16],
            i: 0,
            sp: 0,
            pc: 0x200,
            stack: [0; 16],
            memory: [0; 4096],
            display: Display::new(),
        }
    }

    pub fn from<P: AsRef<Path>>(path: &P) -> Result<Chip8, Box<dyn error::Error>> {
        let mut chip8 = Chip8::new();

        chip8.load_rom(path)?;

        Ok(chip8)
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: &P) -> Result<(), Box<dyn error::Error>> {
        let data = std::fs::read(path)?;

        if data.len() >= self.memory.len() - PROGRAM_START {
            return Err("ROM  bigger than 3583 bytes.")?;
        }

        for i in 0..data.len() {
            self.memory[PROGRAM_START + i] = data[i];
        };

        Ok(())
    }

    fn write_byte(&mut self, index: usize, value: u8) {
        self.memory[index] = value;
    }

    fn read_byte(&self, index: usize) {
        self.memory[index];
    }

    /// Run one fetch-decode-execute cycle
    pub fn run(&mut self) {
        // Fetch
        let opcode: u16 = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[(self.pc + 1) as usize] as u16);

        let op1 = (opcode >> 12 & 0xF) as usize;
        let x = (opcode >> 8 & 0xF) as usize;
        let y = (opcode >> 4 & 0xF) as usize;
        let op2 = (opcode & 0xF) as usize;

        let addr = ((x << 8) | (y << 4) | (op2)) as u16;
        let byte = ((y << 4) | (op2)) as u8;
        
        println!("{}", self.disassemble_op(opcode));

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
                assert!(self.sp >= 0, "RET -> returned with empty stack");

                print!("{}", self.stack[1]);
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            // this would execute a machine language subroutine written at address NNN
            //(0, _, _, _) => format!("SYS  {:X} \t // Would call machine code", addr),
            // JMP addr
            //  Jump to address
            (0x1, _, _, _) => self.pc = addr,
            // CALL addr
            //  Call subroutine at address
            (0x2, _, _, _) => {
                assert!(self.sp < 15, "CALL -> called subroutine with full stack");

                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;

                self.pc = addr;
            }
            // SE Vx, byte
            //  Skip next instruction if Vx = byte
            (0x3, _, _, _) => {
                if self.v[x] == byte {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            //  Skip next instruction if Vx != byte
            (0x4, _, _, _) => {
                if self.v[x] != byte {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            //  Skip next instruction if Vx = Vy
            (0x5, _, _, 0x0) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
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
            }
            // SUB Vx, Vy
            //  Vx = Vx - Vy
            //  VF = Vx > Vy ? 1 : 0
            (0x8, _, _, 0x5) => {
                self.v[x] = self.v[x].overflowing_sub(self.v[y]).0;
                self.v[0xF] = (self.v[x] > self.v[y]) as u8;
            }
            // SHR Vx {, Vy}
            //  set VF = LSB of Vx
            //  Vx = Vx >> 1
            (0x8, _, _, 0x6) => {
                self.v[0xF] = self.v[x] & 0x1;
                self.v[x] >>= 1;
            }
            // SUBN Vx, Vy
            //  Vx = Vy - Vx
            //  VF = Vy > Vx ? 1 : 0
            (0x8, _, _, 0x7) => {
                self.v[x] = self.v[y].overflowing_sub(self.v[x]).0;
                self.v[0xF] = (self.v[y] > self.v[x]) as u8;
            }
            // SHL Vx {, Vy}
            //  VF = MSB of Vx
            //  Vx = Vx << 1;
            (0x8, _, _, 0xE) => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            }
            // SNE Vx, Vy
            //  Skip next instruction if Vx != Vy
            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
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
            }
            // DRW Vx, Vy, nibble
            //  Display n-byte sprite starting at memory location I at (Vx, Vy)
            //  VF = collision
            //  Sprites are XORed onto the existing screen -> if it causes pixels to be erased set VF
            //      The starting position of the sprite will wrap -> X %= 64
            //      The drawing itself does not wrap
            (0xD, _, _, _) => {
                let (x, y) = (self.v[x], self.v[y]);

                // for `op2` bytes in memory
                for row in 0..op2 {
                    if y as usize + row > display::HEIGHT {
                        break;
                    }
                    // get sprite byte from memory
                    let byte = self.memory[self.i as usize];

                    // for bit in byte
                    for bit in 0..8 {
                        if (x + bit) as usize > display::WIDTH {
                            break;
                        }
                        //let pixel = ((byte >> 7 - bit) & 1) == 1;
                        //self.display.screen[y as usize][(x + bit) as usize] ^= pixel;
                        // if bit of sprite is set -> toggle bit at screen[x, y]
                        if ((byte >> (7 - bit)) & 1) == 1 {
                            self.v[0xF] = self.display.toggle(x as usize, y as usize) as u8;
                        }
                    }
                }
            }
            /*
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
            */
            _ => unimplemented!("OPCODE not implemented: {:04X}", opcode),
        };

        self.pc += 2;
    }
    
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
