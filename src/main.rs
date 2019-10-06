use clap::{App, Arg, SubCommand};

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use cursive::views::TextView;
use cursive::Cursive;

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

    let mut siv = Cursive::default();
    let mut data = String::new();

    siv.add_global_callback('q', |s| s.quit());

    data.push_str("ADDR\tBYTE\n");
    //
    for (i, b) in buffer.iter().enumerate() {
        data.push_str(&format!("{:02X}\t{:02X}\n", i, b));
    }
    siv.add_layer(TextView::new(data));

    siv.run();
}
