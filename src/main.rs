#![allow(dead_code, unused_imports, unused_variables, non_snake_case)]

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
            "{:04X} - {}\t{}\t{}",
            self.opcode, self.name, self.description, self.decoded
        )
    }
}

fn parse_nnn(opcode: Word) -> Word {
    let nnn = opcode & 0x0FFF;
    nnn
}

fn parse_xnn(opcode: Word) -> (Word, Word) {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    (x, nn)
}
fn parse_xy0(opcode: Word) -> (Word, Word) {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    (x, y)
}

fn parse_xyn(opcode: Word) -> (Word, Word, Word) {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let n = opcode & 0x000F;
    (x, y, n)
}

fn x0000(opcode: Word) -> Instruction {
    match opcode & 0x00EE {
        0x00EE => return x00EE(opcode),
        _ => {}
    };
    match opcode & 0x00E0 {
        0x00E0 => return x00E0(opcode),
        _ => {}
    };
    let nnn = parse_nnn(opcode);
    let decoded = format!("#{:X}", nnn);
    Instruction::new(opcode, "0NNN", "EXECUTE NNN", decoded)
}

fn x00E0(opcode: Word) -> Instruction {
    let decoded = format!("CLS");
    Instruction::new(opcode, "00E0", "CLEAR SCREEN", decoded)
}

fn x00EE(opcode: Word) -> Instruction {
    let decoded = format!("RETURN");
    Instruction::new(opcode, "00EE", "RETURN FROM SUBROUTINE", decoded)
}

fn x1NNN(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("JMP {}", nnn);
    Instruction::new(opcode, "1NNN", "JUMP TO NNN", decoded)
}

fn x2NNN(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("CALL {:X}", nnn);
    Instruction::new(opcode, "2NNN", "CALL NNN", decoded)
}

fn x3XNN(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}=={:X}", x, nn);
    Instruction::new(opcode, "3XNN", "SKIPIF VX==NN", decoded)
}

fn x4XNN(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}!={:X}", x, nn);
    Instruction::new(opcode, "4XNN", "SKIPIF VX!=NN", decoded)
}

fn x5XY0(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("V{:X}=V{:X}", x, y);
    Instruction::new(opcode, "5XY0", "SKIPIF VX=VY", decoded)
}

fn x6XNN(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}={:X}", x, nn);
    Instruction::new(opcode, "6XNN", "SET VX=NN", decoded)
}

fn x7XNN(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}+={:X}", x, nn);
    Instruction::new(opcode, "7XNN", "ADD VX=VX+NN", decoded)
}

fn x8000(opcode: Word) -> Instruction {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let decoded = format!("V{:X}=V{:X}", x, y);
    Instruction::new(opcode, "8XY0", "LD VX TO VY", decoded)
}

fn x9XY0(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("V{:X}!=V{:X}", x, y);
    Instruction::new(opcode, "9XY0", "SKIPIF VX!=VY", decoded)
}

fn xANNN(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("I={:X}", nnn);
    Instruction::new(opcode, "ANNN", "SET I=NNN", decoded)
}

fn xCXNN(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}=RAND & {:X}", x, nn);
    Instruction::new(opcode, "CXNN", "VX=RAND() & NN", decoded)
}

fn xd000(opcode: Word) -> Instruction {
    let (x, y, n) = parse_xyn(opcode);
    let decoded = format!("V{:X}, V{:X}, N{:X}", x, y, n);
    Instruction::new(opcode, "DXYN", "DRAW VX, VY, N", decoded)
}

fn decode(opcode: Word) -> Instruction {
    match opcode & 0xF000 {
        0x0000 => x0000(opcode),
        0x1000 => x1NNN(opcode),
        0x2000 => x2NNN(opcode),
        0x3000 => x3XNN(opcode),
        0x4000 => x4XNN(opcode),
        0x5000 => x5XY0(opcode),
        0x6000 => x6XNN(opcode),
        0x7000 => x7XNN(opcode),
        0x8000 => x8000(opcode),
        0x9000 => x9XY0(opcode),
        0xA000 => xANNN(opcode),
        0xC000 => xCXNN(opcode),
        0xD000 => xd000(opcode),
        _ => Instruction::new(opcode, "NOP", "UNKOWN OP", "UNKNOWN".to_string()),
    }
}
