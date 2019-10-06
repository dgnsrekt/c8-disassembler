#![allow(dead_code, unused_imports, unused_variables)]

use clap::{App, Arg, SubCommand};

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type Byte = u8;
type Word = u16;
type Address = usize;

fn main() {
    let matches = App::new("c8d")
        .version("1.0")
        .author("dgnsrekt <dgnsrekt@pm.me>")
        .about("Dissassembles chip 8 roms.")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input rom to use")
                .required(true),
        )
        .get_matches();

    // Same as previous example...
    let path = Path::new(matches.value_of("INPUT").unwrap());
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();

    let (x, y): (Vec<(usize, &u8)>, Vec<(usize, &u8)>) =
        buffer.iter().enumerate().partition(|(i, x)| i % 2 == 0);

    let d = x
        .iter()
        .zip(y.iter())
        .map(|((i, a), (_, b))| (i, (**a as u16) << 8 | (**b as u16)))
        .map(|(address, instruction)| format!("0x{:04X} {}", address, decode(instruction)));

    println!("ADDR   INST - TYPE\tDESCRPTION\tINFO");

    d.for_each(|i| println!("{}", i));
}

#[derive(Debug)]
struct Instruction {
    opcode: Word,
    name: String,
    description: String,
    decoded: String,
}

impl Instruction {
    fn new(opcode: Word, name: &str, description: &str, decoded: String) -> Instruction {
        Instruction {
            opcode: opcode,
            name: name.to_string(),
            description: description.to_string(),
            decoded: decoded,
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:X} - {}\t{}\t{}",
            self.opcode, self.name, self.description, self.decoded
        )
    }
}

fn x1000(opcode: Word) -> Instruction {
    let nnn = opcode & 0x0FFF;
    let decoded = format!("#{:X}", nnn);
    Instruction::new(opcode, "1NNN", "JUMP TO NNN", decoded)
}

fn x2000(opcode: Word) -> Instruction {
    let nnn = (opcode & 0x0FFF) << 4;
    let decoded = format!("#{:X}", nnn);
    Instruction::new(opcode, "2NNN", "CALL NNN", decoded)
}

fn x3000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    let decoded = format!("V{:X}={:02X}", x, nn);
    Instruction::new(opcode, "3XNN", "SKIPIF VX==NN", decoded)
}

fn x6000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    let decoded = format!("V{:X}={:02X}", x, nn);
    Instruction::new(opcode, "6XNN", "SET VX=NN", decoded)
}

fn x7000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    let decoded = format!("V{:X}+={:02X}", x, nn);
    Instruction::new(opcode, "7XNN", "ADD VX=VX+NN", decoded)
}

fn x8000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let decoded = format!("V{:X}=V{:X}", x, y);
    Instruction::new(opcode, "8XY0", "LD VX TO VY", decoded)
}

fn xa000(opcode: Word) -> Instruction {
    let nnn = opcode & 0x0FFF;
    let decoded = format!("#{:X}", nnn);
    Instruction::new(opcode, "ANNN", "SET I=NNN", decoded)
}

fn xc000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    let decoded = format!("V{:X} & {:02X}", x, nn);
    Instruction::new(opcode, "CXNN", "VX=RAND() & NN", decoded)
}

fn xd000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let n = opcode & 0x000F;
    let decoded = format!("V{:X}, V{:X}, N{:X}", x, y, n);
    Instruction::new(opcode, "DXYN", "DRAW VX, VY, N", decoded)
}

fn decode(opcode: Word) -> Instruction {
    match opcode & 0xF000 {
        0x1000 => x1000(opcode),
        0x2000 => x2000(opcode),
        0x3000 => x3000(opcode),
        0x6000 => x6000(opcode),
        0x7000 => x7000(opcode),
        0x8000 => x8000(opcode),
        0xA000 => xa000(opcode),
        0xC000 => xc000(opcode),
        0xD000 => xd000(opcode),
        _ => Instruction::new(opcode, "NOP", "UNKOWN OP", "UNKNOWN".to_string()),
    }
}
