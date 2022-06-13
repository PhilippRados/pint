#![allow(unused)]
use clap::Parser;
use std::fs::File;

use decoder::decode_png;
use decoder::is_valid_png;
mod decoder;

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn main() {
    let args = Cli::parse();
    let mut file = match File::open(args.path) {
        Err(why) => {
            eprintln!("pint: couldn't open file: {}", why);
            std::process::exit(1);
        }
        Ok(val) => val,
    };

    is_valid_png(&mut file);
    let rgb_img = decode_png(file);
    // TODO interpret
}
