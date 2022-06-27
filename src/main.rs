#![allow(unused)]
use std::cmp;
use std::fs::File;

#[macro_use]
extern crate enum_index_derive;

use decoder::RGB;
use enum_index::EnumIndex;
mod cli_options;
mod decoder;

#[derive(EnumIndex)]
enum Direction {
    // maybe as hashmap
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

fn furthest_dp_direction(dp: &Direction, block: &Vec<Coordinates>) -> Vec<Coordinates> {
    match *dp {
        Direction::RIGHT => block
            .iter()
            .filter(|pos| pos.x == block.iter().max_by_key(|p| p.x).unwrap().x)
            .cloned()
            .collect(),
        Direction::DOWN => block
            .iter()
            .filter(|pos| pos.y == block.iter().max_by_key(|p| p.y).unwrap().y)
            .cloned()
            .collect(),
        Direction::LEFT => block
            .iter()
            .filter(|pos| pos.x == block.iter().min_by_key(|p| p.x).unwrap().x)
            .cloned()
            .collect(),
        Direction::UP => block
            .iter()
            .filter(|pos| pos.y == block.iter().min_by_key(|p| p.y).unwrap().y)
            .cloned()
            .collect(),
    }
}

fn next_pos(dp: &Direction, cc: &CodelChooser, block: &Vec<Coordinates>) -> Coordinates {
    let block_edge = furthest_dp_direction(dp, block);
    Coordinates { x: 0, y: 0 }
}
fn remove_all<T: Eq>(arr: &mut Vec<T>, element: &T) {
    arr.retain(|e| e != element);
}

fn in_range(new_pos: &Coordinates, rgb_img: &Vec<Vec<RGB>>) -> bool {
    let width = rgb_img[0].len() as i32;
    let height = rgb_img.len() as i32;
    let x_pos = new_pos.x;
    let y_pos = new_pos.y;

    if x_pos < width && x_pos >= 0 && y_pos < height && y_pos >= 0 {
        true
    } else {
        false
    }
}

fn is_color(new_pos: &Coordinates, rgb_img: &Vec<Vec<RGB>>, color: RGB) -> bool {
    rgb_img[new_pos.y as usize][new_pos.x as usize] == color
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
        let new_pos = Coordinates {
            x: current_pos.x + (direction.x * codel_size),
            y: current_pos.y + (direction.y * codel_size),
        };

        if in_range(&new_pos, &rgb_img)
            && is_color(&new_pos, &rgb_img, color)
            && !counted.contains(&new_pos)
        {
            not_counted.push(new_pos);
        }
    }
}

fn get_block(rgb_img: &Vec<Vec<RGB>>, pos: Coordinates, codel_size: i32) -> Vec<Coordinates> {
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
    counted
}

fn get_size(block: &Vec<Coordinates>) -> i32 {
    block.len() as i32
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
    let mut block: Vec<Coordinates>;

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

    loop {
        block = get_block(&rgb_img, pos, codel_size);
        block_size = get_size(&block);
        let prev = pos;
        pos = next_pos(&dp, &cc, &block);
        //interprete(prev,pos);

        println!("{:?}", rgb_img[pos.y as usize][pos.x as usize]);
    }

    // white works as comment
    // black toggles codelchooser if afterwards still black move dp clockwise
    // when full rotation in same color block => exit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_correct_block_size_red() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let block = get_block(&rgb_img, Coordinates { x: 0, y: 0 }, 5);
        let result = get_size(&block);

        assert_eq!(result, 72);
    }
    #[test]
    fn get_correct_block_size_pink() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let block = get_block(&rgb_img, Coordinates { x: 60, y: 0 }, 5);
        let result = get_size(&block);
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

    #[test]
    fn furthest_dp_direction_right() {
        let block = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 50 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        let dp = Direction::RIGHT;

        let result = furthest_dp_direction(&dp, &block);
        let expected = vec![
            Coordinates { x: 25, y: 50 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn furthest_dp_direction_up() {
        let block = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 0 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        let dp = Direction::UP;

        let result = furthest_dp_direction(&dp, &block);
        let expected = vec![Coordinates { x: 0, y: 0 }, Coordinates { x: 25, y: 0 }];
        assert_eq!(result, expected);
    }
}
