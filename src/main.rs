use std::fs::File;
use std::io;

fn main() {
    let rom = std::fs::read("/home/tep/dev/chippy/roms/games/Cave.ch8").unwrap();
    let mut pc: u16 = 0;

    loop {
        disassemble_op(&rom, pc as usize);
        pc += 1;
    }
}

fn disassemble_op(rom: &Vec<u8>, pc: usize) {
    let opcode: u16 = (rom[pc] as u16) << 8 | (rom[pc+1] as u16);

    let op1 = opcode >> 12 & 0xF;
    let x =   opcode >>  8 & 0xF;
    let y =   opcode >>  4 & 0xF;
    let op2 = opcode       & 0xF;

    match (op1, x, y, op2) {
        (0, 0, 0xE, 0) => println!("CLS"),
        _ => unimplemented!("OPCODE not implemented: {:#04x}", opcode),
    }
}