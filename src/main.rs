#![allow(unused)]
use std::fs::File;

use navigation::*;
use types::*;
mod cli_options;
mod decoder;
mod navigation;
mod types;

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

fn get_color_index(color: RGB) -> Option<Coordinates> {
    for (y, dark_arr) in COLORS.iter().enumerate() {
        for x in 0..dark_arr.len() {
            if color == dark_arr[x] {
                return Some(Coordinates {
                    x: x as i32,
                    y: y as i32,
                });
            }
        }
    }
    None
}

fn calculate_color_diff(prev_color: RGB, color: RGB) -> Coordinates {
    // get color indices and calculate diff
    let prev = get_color_index(prev_color).unwrap();
    let current = get_color_index(color).unwrap();

    Coordinates {
        x: (current.x - prev.x).rem_euclid(6), // basically pythons () % 6
        y: (current.y - prev.y).rem_euclid(3),
    }
}

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

    // const LIGHT: [&str; 3] = ["light", "normal", "dark"];
    // const HUE: [&str; 6] = ["red", "yellow", "green", "cyan", "blue", "magenta"];

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

        calculate_color_diff(prev_color.color, color.color);
        // interprete(prev_color, new_color, &stack);
    }
    // TODO implement white blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_correct_color_index() {
        let color = RGB(0, 255, 0);
        let result = get_color_index(color).unwrap();
        let expected = Coordinates { x: 2, y: 1 };

        assert_eq!(result, expected);
    }

    #[test]
    fn gets_correct_color_diff_1() {
        let prev = RGB(0, 255, 0);
        let current = RGB(255, 192, 192);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 4, y: 2 };

        assert_eq!(result, expected);
    }

    #[test]
    fn gets_correct_color_diff_2() {
        let prev = RGB(192, 255, 192);
        let current = RGB(192, 255, 255);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 1, y: 0 };

        assert_eq!(result, expected);
    }
    #[test]
    fn gets_correct_color_diff_3() {
        let prev = RGB(192, 0, 192);
        let current = RGB(255, 0, 255);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 0, y: 2 };

        assert_eq!(result, expected);
    }
}
