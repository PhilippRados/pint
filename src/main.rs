#![allow(unused)]
use std::fs::File;

use cli_options::cli_options;
use decoder::check_valid_png;
use decoder::decode_png;
mod cli_options;
mod decoder;

fn main() {
    let opt = cli_options();

    let path = opt.value_of("file").unwrap();
    let mut file = match File::open(path) {
        Err(why) => {
            eprintln!("pint: couldn't open file: {}", why);
            std::process::exit(1);
        }
        Ok(val) => val,
    };

    check_valid_png(&mut file);
    let rgb_img = decode_png(file);

    // TODO interpret
}
