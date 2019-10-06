#![allow(dead_code, unused_imports, unused_variables)]

use clap::{App, Arg, SubCommand};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use cursive::align::HAlign;
use cursive::views::{
    Dialog, LinearLayout, ListView, ScrollView, SelectView, SliderView, TextView, ViewBox,
};
use cursive::Cursive;

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

    //let mut siv = Cursive::default();
    //let mut sv = ScrollView::new().h_align(HAlign::Left);
    //let mut sv = SelectView::new();

    //siv.add_global_callback('q', |s| s.quit());

    // for (i, b) in buffer.iter().enumerate().step_by(2) {
    //     let y = format!("0x{:02X}\t{:02X}{:02X}", i + 0x200, b, buffer[i + 1]);
    //     data.add_item(y, i);
    // }
    // let x = buffer.iter().step_by(2);
    // let y = buffer.iter().skip(1).step_by(2);
    // let z = x.zip(y);
    // for (i, g) in z.enumerate() {
    //     println!("{:02X}\t{:02X?}", i, g);
    // }

    let (x, y): (Vec<(usize, &u8)>, Vec<(usize, &u8)>) =
        buffer.iter().enumerate().partition(|(i, x)| i % 2 == 0);

    x.iter()
        .zip(y.iter())
        .map(|((i, a), (_, b))| (i, (**a as u16) << 8 | (**b as u16)))
        .for_each(|x| decode(x));

    //println!("{:?}", y.iter());

    //sv.add_all(u);

    // data.set_on_submit(|addr, opcode| {
    //     //addr.pop_layer();
    //     let text = format!("decoding {:02X}...", opcode);
    //     //addr.add_layer(Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()));
    //     addr.add_layer(TextView::new(text));
    // });
    //siv.add_layer(Dialog::around(data).title("ADDR\tBYTE"));
    //siv.add_layer(sv);

    //siv.run();
}

fn x1000(addr: Address, opcode: Word) {
    let nnn = opcode & 0x0FFF;
    println!(
        "0x{:02X} {:X} - 1NNN\tJUMP TO NNN\t#{:X}",
        addr, opcode, nnn
    );
}

fn x2000(addr: Address, opcode: Word) {
    let nnn = (opcode & 0x0FFF) << 4;
    println!("0x{:02X} {:X} - 2NNN\tCALL NNN\t#{:X}", addr, opcode, nnn);
}

fn x3000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    println!(
        "0x{:02X} {:X} - 3XNN\tSKIPIF VX=NN\tV{:X}={:02X}",
        addr, opcode, x, nn
    );
}

fn x6000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    println!(
        "0x{:02X} {:X} - 6XNN\tSET VX=NN\tV{:X}={:02X}",
        addr, opcode, x, nn,
    );
}

fn x7000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    println!(
        "0x{:02X} {:X} - 7XNN\tADD VX=VX+NN\tV{3:X}=V{3:X}+{:02X}",
        addr, opcode, x, nn
    );
}

fn x8000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    println!(
        "0x{:02X} {:X} - 8XY0\tLD VX TO VY\tV{:X}=V{:X}",
        addr, opcode, x, y
    );
}

fn xa000(addr: Address, opcode: Word) {
    let nnn = opcode & 0x0FFF;
    println!("0x{:02X} {:X} - ANNN\tSET I=NNN\t#{:X}", addr, opcode, nnn);
}

fn xc000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let nn = opcode & 0x00FF;
    println!(
        "0x{:02X} {:X} - CXNN\tVX = RAND & NN\tV{:X} & {:02X}",
        addr, opcode, x, nn
    );
}

fn xd000(addr: Address, opcode: Word) {
    let x = (opcode & 0x0F00) >> 8;
    let y = (opcode & 0x00F0) >> 4;
    let n = opcode & 0x000F;
    println!(
        "0x{:02X} {:X} - DXYN\tDRAW VX, VY, N\tV{:X},V{:X},N{:X}",
        addr, opcode, x, y, n
    );
}

fn decode(x: (&usize, Word)) {
    let (addr, opcode) = x;
    let addr = addr + 0x200;

    match opcode & 0xF000 {
        0x1000 => x1000(addr, opcode),
        0x2000 => x2000(addr, opcode),
        0x3000 => x3000(addr, opcode),
        0x6000 => x6000(addr, opcode),
        0x7000 => x7000(addr, opcode),
        0x8000 => x8000(addr, opcode),
        0xA000 => xa000(addr, opcode),
        0xC000 => xc000(addr, opcode),
        0xD000 => xd000(addr, opcode),
        _ => println!("{:02X?}", opcode),
    }
    //println!("decoded {:02X?}", x);
}
