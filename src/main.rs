use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let roms = fs::read_dir("/home/tep/dev/chippy/roms")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    for rom in roms {
        if !rom.is_file() {
            continue
        }

        println!("{}", rom.to_string_lossy());
        let rom = std::fs::read(rom)?;
        let mut pc: u16 = 0;

        loop {
            if pc as usize >= rom.len()-3 {
                break;
            }
            //println!("{}", disassemble_op(&rom, pc as usize));
            disassemble_op(&rom, pc as usize);
            pc += 2;
        }
    }

    Ok(())
}

fn disassemble_op(rom: &Vec<u8>, pc: usize) -> String {
    let opcode: u16 = (rom[pc] as u16) << 8 | (rom[pc+1] as u16);

    let op1 = opcode >> 12 & 0xF;
    let x =   opcode >>  8 & 0xF;
    let y =   opcode >>  4 & 0xF;
    let op2 = opcode       & 0xF;

    let addr = (x << 8) | (y << 4) | (op2);
    let byte = (y << 4) | (op2);

    let decompiled = match (op1, x, y, op2) {
        (0x0, 0x0, 0xE, 0x0) => String::from("CLS"),
        (0x0, 0x0, 0xE, 0xE) => String::from("RET"),
        // this would execute a machine language subroutine written at address NNN
        (0, _, _, _)         => format!("SYS  {:X} \t // Would call machine code", addr),
        (0x1, _, _, _)       => format!("JMP  {:X}", addr),
        (0x2, _, _, _)       => format!("CALL {:X}", addr),
        (0x3, _, _, _)       => format!("SE   V{:X}, {:X}", x, byte),
        (0x4, _, _, _)       => format!("SNE  V{:X}, {:X}", x, byte),
        (0x5, _, _, 0x0)     => format!("SE   V{:X}, V{:X}", x, y),
        (0x6, _, _, _)       => format!("LD   V{:X}, {:X}", x, byte),
        (0x7, _, _, _)       => format!("ADD  V{:X}, {:X}", x, byte),
        (0x8, _, _, 0x0)     => format!("LD   V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x1)     => format!("OR   V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x2)     => format!("AND  V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x3)     => format!("XOR  V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x4)     => format!("ADD  V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x5)     => format!("SUB  V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x6)     => format!("SHR  V{:X}, V{:X}", x, y),
        (0x8, _, _, 0x7)     => format!("SUBN V{:X}, V{:X}", x, y),
        (0x8, _, _, 0xE)     => format!("SHL  V{:X}, V{:X}", x, y),
        (0x9, _, _, 0x0)     => format!("SNE  V{:X}, V{:X}", x, y),
        (0xA, _, _, _)       => format!("LD   I, {:X}", addr),
        (0xB, _, _, _)       => format!("JP   V0, {:X}", addr),
        (0xC, _, _, _)       => format!("RND  V{:X}, {:X}", x, byte),
        (0xD, _, _, _)       => format!("DRW  V{:X}, V{:X}, {:X}", x, y, op2),
        (0xE, _, 0x9, 0xE)   => format!("SKP  V{:X}", x),
        (0xE, _, 0xA, 0x1)   => format!("SKNP V{:X}", x),
        (0xF, _, 0x0, 0x7)   => format!("LD   V{:X}, DT", x),
        (0xF, _, 0x0, 0xA)   => format!("LD   V{:X}, K", x),
        (0xF, _, 0x1, 0x5)   => format!("LD   DT, V{:X}", x),
        (0xF, _, 0x1, 0x8)   => format!("LD   ST, V{:X}", x),
        (0xF, _, 0x1, 0xE)   => format!("ADD  I, V{:X}", x),
        (0xF, _, 0x2, 0x9)   => format!("LD   F, V{:X}", x),
        (0xF, _, 0x3, 0x3)   => format!("LD   B, V{:X}", x),
        (0xF, _, 0x5, 0x5)   => format!("LD   [I], V{:X}", x),
        (0xF, _, 0x6, 0x5)   => format!("LD   V{:X}, [I]", x),
        _ => unimplemented!("OPCODE not implemented: {:04X}", opcode),
    };

    return format!("[{:04X}] {:04X} | {}", pc, opcode, decompiled)
}