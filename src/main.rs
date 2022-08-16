use std::fs::File;

use pint::cli_options::*;
use pint::decoder::*;
use pint::interpreter::*;
use pint::navigation::*;
use pint::types::*;

fn main() {
    let opt = cli_options();

    let mut codel_size = match opt.value_of("codel_size") {
        Some(v) => v.parse::<i32>().unwrap(),
        None => -1,
    };
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
    if codel_size == -1 {
        codel_size = infer_codel_size(&rgb_img);
    }

    let mut dp = Direction::RIGHT;
    let mut cc = CodelChooser::LEFT;
    let mut pos = Coordinates { x: 0, y: 0 };

    let mut stack = Vec::new();
    let mut current_color = ColorInfo {
        color: rgb_img[pos.y as usize][pos.x as usize],
        size: get_size(&get_block(&rgb_img, pos, codel_size, dp)),
    };
    loop {
        let prev_color = current_color;
        current_color = match next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc) {
            Some(new_color) => new_color,
            None => break,
        };
        execute(&mut stack, &mut dp, &mut cc, prev_color, &current_color);
    }
}
