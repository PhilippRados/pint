#![allow(unused)]
use std::fs::File;

use cli_options::cli_options;
use decoder::check_valid_png;
use decoder::decode_png;
mod cli_options;
mod decoder;

enum Direction {
    RIGHT,
    DOWN,
    LEFT,
    UP,
}

impl Direction {
    fn cords(&self) -> Coordinates {
        match self {
            Self::RIGHT => Coordinates { x: 1, y: 0 },
            Self::DOWN => Coordinates { x: 0, y: 1 },
            Self::LEFT => Coordinates { x: -1, y: 0 },
            Self::UP => Coordinates { x: 0, y: -1 },
        }
    }
}

struct Coordinates {
    x: i32,
    y: i32,
}

enum CodelChooser {
    LEFT,
    RIGHT,
}

fn next_pos(coordinates: Coordinates, dp: &Direction, codel_size: i32) -> Coordinates {
    Coordinates {
        x: coordinates.x + (codel_size * dp.cords().x),
        y: coordinates.y + (codel_size * dp.cords().y),
    }
}

fn main() {
    let opt = cli_options();

    let codel_size = opt.value_of("codel_size").unwrap().parse::<i32>().unwrap();
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

    let mut dp = Direction::RIGHT;
    let mut cc = CodelChooser::LEFT;
    let mut cor = Coordinates { x: 0, y: 0 };
    // loop {
    //     cor = next_pos(cor, &dp, codel_size);
    // }

    // white works as comment
    // black toggles codelchooser if afterwards still black move dp clockwise
}
