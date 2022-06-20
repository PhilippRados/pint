#![allow(unused)]
use std::fs::File;

#[macro_use]
extern crate enum_index_derive;

use decoder::RGB;
use enum_index::EnumIndex;
mod cli_options;
mod decoder;

#[derive(EnumIndex)]
enum Direction {
    RIGHT,
    DOWN,
    LEFT,
    UP,
}

const CORDS: [Coordinates; 4] = [
    Coordinates { x: 1, y: 0 },  //RIGHT
    Coordinates { x: 0, y: 1 },  // DOWN
    Coordinates { x: -1, y: 0 }, // LEFT
    Coordinates { x: 0, y: -1 }, // UP
];

impl Direction {
    fn cords(&self) -> Coordinates {
        CORDS[self.enum_index()]
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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
fn remove_all<T: Eq>(arr: &mut Vec<T>, element: &T) {
    arr.retain(|e| e != element);
}

fn in_range(x_pos: i32, y_pos: i32, rgb_img: &Vec<Vec<RGB>>) -> bool {
    let width = rgb_img[0].len() as i32;
    let height = rgb_img.len() as i32;

    if x_pos < width && x_pos >= 0 && y_pos < height && y_pos >= 0 {
        true
    } else {
        false
    }
}

fn is_color(x_pos: i32, y_pos: i32, rgb_img: &Vec<Vec<RGB>>, color: RGB) -> bool {
    rgb_img[y_pos as usize][x_pos as usize] == color
}

fn check_adjacent_codels(
    current_pos: Coordinates,
    codel_size: i32,
    rgb_img: &Vec<Vec<RGB>>,
    counted: &mut Vec<Coordinates>,
    not_counted: &mut Vec<Coordinates>,
    color: RGB,
) {
    for direction in CORDS {
        let x_pos = current_pos.x + (direction.x * codel_size);
        let y_pos = current_pos.y + (direction.y * codel_size);

        if in_range(x_pos, y_pos, &rgb_img)
            && is_color(x_pos, y_pos, &rgb_img, color)
            && !counted.contains(&Coordinates { x: x_pos, y: y_pos })
        {
            not_counted.push(Coordinates { x: x_pos, y: y_pos });
        }
    }
}

fn get_block_size(rgb_img: Vec<Vec<RGB>>, pos: Coordinates, codel_size: i32) -> i32 {
    let mut counted: Vec<Coordinates> = Vec::new();
    let mut not_counted: Vec<Coordinates> = Vec::new();
    not_counted.push(Coordinates { ..pos });
    let color = rgb_img[pos.y as usize][pos.x as usize];
    let mut current_pos = pos;

    while not_counted.len() > 0 {
        while rgb_img[current_pos.y as usize][current_pos.x as usize] == color {
            if not_counted.contains(&current_pos) {
                // remove from not_counted add to counted
                remove_all::<Coordinates>(&mut not_counted, &current_pos);
                counted.push(current_pos);
            }
            // mark adjacent codels as not_counted
            check_adjacent_codels(
                current_pos,
                codel_size,
                &rgb_img,
                &mut counted,
                &mut not_counted,
                color,
            );

            current_pos.x += codel_size;
        }
        if not_counted.len() > 0 {
            current_pos = not_counted[0];
        } else {
            break;
        };
    }
    counted.len() as i32
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
    let mut block_size = 0;

    // let result = get_block_size(rgb_img, pos, codel_size);
    // loop {
    //     cor = next_pos(cor, &dp, codel_size);
    // }

    // white works as comment
    // black toggles codelchooser if afterwards still black move dp clockwise
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_correct_block_size_red() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let result = get_block_size(rgb_img, Coordinates { x: 0, y: 0 }, 5);
        assert_eq!(result, 72);
    }
    #[test]
    fn get_correct_block_size_pink() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let result = get_block_size(rgb_img, Coordinates { x: 60, y: 0 }, 5);
        assert_eq!(result, 101);
    }
    #[test]
    fn remove_works() {
        let cord = Coordinates { x: 25, y: 50 };
        let mut cords = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 50 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        let correct = vec![Coordinates { x: 0, y: 0 }, Coordinates { x: 25, y: 60 }];

        remove_all::<Coordinates>(&mut cords, &cord);
        assert_eq!(cords, correct);
    }
}
