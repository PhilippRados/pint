// #![allow(unused)]
use std::fs::File;

use navigation::*;
use types::*;
mod cli_options;
mod decoder;
mod navigation;
mod types;

fn main() {
    let opt = cli_options::cli_options();

    let codel_size = opt.value_of("codel_size").unwrap().parse::<i32>().unwrap();
    let path = opt.value_of("file").unwrap();
    let mut file = match File::open(path) {
        Err(why) => {
            eprintln!("pint: couldn't open file: {}", why);
            std::process::exit(1);
        }
        Ok(val) => val,
    };

    decoder::check_valid_png(&mut file);
    let rgb_img = decoder::decode_png(file);

    let mut dp = Direction::RIGHT;
    let mut cc = CodelChooser::LEFT;
    let mut pos = Coordinates { x: 0, y: 0 };

    const LIGHT: [&str; 3] = ["light", "normal", "dark"];
    const HUE: [&str; 6] = ["red", "yellow", "green", "cyan", "blue", "magenta"];
    const COLORS: [[RGB; 6]; 3] = [
        [
            RGB(255, 192, 192),
            RGB(255, 255, 192),
            RGB(192, 255, 192),
            RGB(192, 255, 255),
            RGB(192, 192, 255),
            RGB(255, 192, 255),
        ],
        [
            RGB(255, 0, 0),
            RGB(255, 255, 0),
            RGB(0, 255, 0),
            RGB(0, 255, 255),
            RGB(0, 0, 255),
            RGB(255, 0, 255),
        ],
        [
            RGB(192, 0, 0),
            RGB(192, 192, 0),
            RGB(0, 192, 0),
            RGB(0, 192, 192),
            RGB(0, 0, 192),
            RGB(192, 0, 192),
        ],
    ];

    // let stack = Vec::new();
    let prev_color = ColorInfo {
        color: rgb_img[pos.y as usize][pos.x as usize],
        size: get_size(&get_block(&rgb_img, pos, codel_size)),
    };
    loop {
        let color = match next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc) {
            Some(new_color) => new_color,
            None => break,
        };

        // interprete(prev_color, new_color, &stack);
    }
    // white works as comment
    // black toggles codelchooser if afterwards still black move dp clockwise
    // when full rotation in same color block => exit
}
