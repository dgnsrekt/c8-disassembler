#![allow(dead_code, unused_imports, unused_variables)]
use clap::{App, Arg, SubCommand};

use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

type Byte = u8;
type Word = u16;
type Address = usize;
type Buffer = Vec<u8>;
type Memory = Vec<u8>;

fn open_rom(path: &Path) -> Buffer {
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", &path.display(), why.description()),
        Ok(file) => file,
    };

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    buffer
}

fn main() {
    let matches = App::new("c8d")
        .version("1.0")
        .author("dgnsrekt <dgnsrekt@pm.me>")
        .about("Dissassembles chip 8 roms.")
        .arg(
            Arg::with_name("ROM")
                .help("Sets the input rom to use")
                .required(true),
        )
        .get_matches();

    let rom_path = Path::new(matches.value_of("ROM").unwrap());

    let buffer = open_rom(rom_path);

    let (x, y): (Vec<(usize, &u8)>, Vec<(usize, &u8)>) =
        buffer.iter().enumerate().partition(|(i, x)| i % 2 == 0);

    let memory = x
        .iter()
        .zip(y.iter())
        .map(|((i, a), (_, b))| (i, (**a as u16) << 8 | (**b as u16)))
        .map(|(address, instruction)| format!("0x{:04X} {}", address + 0x200, decode(instruction)));

    println!("ADDR   OP     INST\tDESCRPTION\tINFO");
    println!("-------------------------------------------------");

    memory.for_each(|i| println!("{}", i));

    let stdout = io::stdout().into_raw_mode().unwrap();
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|mut f| {
            let size = f.size();
            Block::default()
                .title("Block")
                .borders(Borders::ALL)
                .render(&mut f, size);
        })
        .unwrap();
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

fn decode_x0000(opcode: Word) -> Instruction {
    match opcode & 0x00EE {
        0x00EE => return decode_x00ee(opcode),
        _ => {}
    };
    match opcode & 0x00E0 {
        0x00E0 => return decode_x00e0(opcode),
        _ => {}
    };
    let nnn = parse_nnn(opcode);
    let decoded = format!("#{:X}", nnn);
    Instruction::new(opcode, "0NNN", "EXECUTE NNN", decoded)
}

fn decode_x00e0(opcode: Word) -> Instruction {
    let decoded = format!("CLS");
    Instruction::new(opcode, "00E0", "CLEAR SCREEN", decoded)
}

fn decode_x00ee(opcode: Word) -> Instruction {
    let decoded = format!("RETURN");
    Instruction::new(opcode, "00EE", "RETURN FROM SUBROUTINE", decoded)
}

fn decode_x1nnn(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("JMP #{:X}", nnn);
    Instruction::new(opcode, "1NNN", "JUMP TO NNN", decoded)
}

fn decode_x2nnn(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("CALL {:X}", nnn);
    Instruction::new(opcode, "2NNN", "CALL NNN", decoded)
}

fn decode_x3xnn(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}=={:X}", x, nn);
    Instruction::new(opcode, "3XNN", "SKIPIF VX==NN", decoded)
}

fn decode_x4xnn(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}!={:X}", x, nn);
    Instruction::new(opcode, "4XNN", "SKIPIF VX!=NN", decoded)
}

fn decode_x5xy0(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("V{:X}=V{:X}", x, y);
    Instruction::new(opcode, "5XY0", "SKIPIF VX=VY", decoded)
}

fn decode_x6xnn(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}={:X}", x, nn);
    Instruction::new(opcode, "6XNN", "SET VX=NN", decoded)
}

fn decode_x7xnn(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}+={:X}", x, nn);
    Instruction::new(opcode, "7XNN", "ADD VX=VX+NN", decoded)
}

fn decode_x8000(opcode: Word) -> Instruction {
    match opcode & 0x000F {
        0x0000 => decode_x8xy0(opcode),
        0x0001 => decode_x8xy1(opcode),
        0x0002 => decode_x8xy2(opcode),
        0x0003 => decode_x8xy3(opcode),
        _ => unimplemented!("opcode not impl for {:04X}.", opcode),
    }
}

fn decode_x8xy0(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("V{:X}=V{:X}", x, y);
    Instruction::new(opcode, "8XY0", "STORE VX IN VY", decoded)
}

fn decode_x8xy1(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("VX=V{:X}|V{:X}", x, y);
    Instruction::new(opcode, "8XY1", "SET VX TO VX|VY", decoded)
}

fn decode_x8xy2(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("VX=V{:X}&V{:X}", x, y);
    Instruction::new(opcode, "8XY2", "SET VX TO VX&VY", decoded)
}

fn decode_x8xy3(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("VX=V{:X}^V{:X}", x, y);
    Instruction::new(opcode, "8XY3", "SET VX TO VX^VY", decoded)
}

fn decode_x9xy0(opcode: Word) -> Instruction {
    let (x, y) = parse_xy0(opcode);
    let decoded = format!("V{:X}!=V{:X}", x, y);
    Instruction::new(opcode, "9XY0", "SKIPIF VX!=VY", decoded)
}

fn decode_xannn(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("I={:X}", nnn);
    Instruction::new(opcode, "ANNN", "SET I=NNN", decoded)
}

fn decode_xbnnn(opcode: Word) -> Instruction {
    let nnn = parse_nnn(opcode);
    let decoded = format!("{}+V0", nnn);
    Instruction::new(opcode, "BNNN", "JUMP TO NNN+V0", decoded)
}
fn decode_xcxnn(opcode: Word) -> Instruction {
    let (x, nn) = parse_xnn(opcode);
    let decoded = format!("V{:X}=RAND & {:X}", x, nn);
    Instruction::new(opcode, "CXNN", "VX=RAND() & NN", decoded)
}

fn decode_xdxyn(opcode: Word) -> Instruction {
    let (x, y, n) = parse_xyn(opcode);
    let decoded = format!("V{:X}, V{:X}, N{:X}", x, y, n);
    Instruction::new(opcode, "DXYN", "DRAW VX, VY, N", decoded)
}

fn decode_xe000(opcode: Word) -> Instruction {
    match opcode & 0x00FF {
        0x009E => decode_xex9e(opcode),
        0x00A1 => decode_xexa1(opcode),
        0x0062 => decode_xexa1(opcode), //TODO: fix
        0x00A5 => decode_xexa1(opcode), //TODO: fix
        0x0082 => decode_xexa1(opcode), //TODO: fix
        0x0028 => decode_xexa1(opcode), //TODO: fix
        _ => unimplemented!("opcode not impl for {:04X}.", opcode),
    }
}

fn decode_xex9e(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("SKIPIF KEY==V{}", x);
    Instruction::new(opcode, "EX9E", "SKIPIF VX==KEY", decoded)
}

fn decode_xexa1(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("SKIPIF KEY=!V{}", x);
    Instruction::new(opcode, "EXA1", "SKIPIF VX=!KEY", decoded)
}

fn decode_xf000(opcode: Word) -> Instruction {
    match opcode & 0x00FF {
        0x0007 => decode_xfx07(opcode),
        0x000A => decode_xfx0a(opcode),
        0x0015 => decode_xfx15(opcode),
        0x0018 => decode_xfx18(opcode),
        0x001E => decode_xfx1e(opcode),
        0x0029 => decode_xfx29(opcode),
        0x00A5 => decode_xfx29(opcode), //TODO: fix
        _ => unimplemented!("opcode not impl for {:04X}.", opcode),
    }
}

fn decode_xfx07(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("V{}=DELAY", x);
    Instruction::new(opcode, "FX07", "STORE VX IN DELAY TIMER", decoded)
}

fn decode_xfx0a(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("V{}=KEY", x);
    Instruction::new(opcode, "FX0A", "STORE VX TO KEY PRESS", decoded)
}

fn decode_xfx15(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("DELAY TIMER=V{}", x);
    Instruction::new(opcode, "FX15", "SET DELAY TO VX", decoded)
}

fn decode_xfx18(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("SOUND TIMER=V{}", x);
    Instruction::new(opcode, "FX18", "SET SOUND TO VX", decoded)
}

fn decode_xfx1e(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("V{}+I", x);
    Instruction::new(opcode, "FX1E", "ADD VX TO I", decoded)
}

fn decode_xfx29(opcode: Word) -> Instruction {
    let (x, _) = parse_xy0(opcode);
    let decoded = format!("SET I, V{}", x);
    Instruction::new(opcode, "FX29", "SET I TO HEX AT VX", decoded)
}

fn decode(opcode: Word) -> Instruction {
    match opcode & 0xF000 {
        0x0000 => decode_x0000(opcode),
        0x1000 => decode_x1nnn(opcode),
        0x2000 => decode_x2nnn(opcode),
        0x3000 => decode_x3xnn(opcode),
        0x4000 => decode_x4xnn(opcode),
        0x5000 => decode_x5xy0(opcode),
        0x6000 => decode_x6xnn(opcode),
        0x7000 => decode_x7xnn(opcode),
        0x8000 => decode_x8000(opcode),
        0x9000 => decode_x9xy0(opcode),
        0xA000 => decode_xannn(opcode),
        0xB000 => decode_xbnnn(opcode),
        0xC000 => decode_xcxnn(opcode),
        0xD000 => decode_xdxyn(opcode),
        0xE000 => decode_xe000(opcode),
        0xF000 => decode_xf000(opcode),
        _ => unimplemented!("opcode not impl for {:04X}.", opcode),
        //Instruction::new(opcode, "NOP", "UNKOWN OP", "UNKNOWN".to_string()),
    }
}
