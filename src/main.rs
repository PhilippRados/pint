#![allow(unused)]
use std::fs::File;

use cli_options::cli_options;
use decoder::check_valid_png;
use decoder::decode_png;
use decoder::RGB;
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

fn get_block_size(rgb_img: Vec<Vec<RGB>>, pos: Coordinates, codel_size: i32) -> i32 {
    let mut counted: Vec<Coordinates> = Vec::new();
    let mut not_counted: Vec<Coordinates> = Vec::new();
    not_counted.push(Coordinates { ..pos });
    let color = rgb_img[pos.y as usize][pos.x as usize];
    let mut current_pos = pos;

    while not_counted.len() > 0 {
        while rgb_img[current_pos.y as usize][current_pos.x as usize] == color {
            if not_counted.contains(&current_pos) {
                // remove from not_Counted add to counted
                remove_all::<Coordinates>(&mut not_counted, &current_pos);
                counted.push(current_pos);
            }

            if current_pos.x + codel_size < rgb_img[current_pos.y as usize].len() as i32
                && rgb_img[current_pos.y as usize][(current_pos.x + codel_size) as usize] == color
                && !counted.contains(&Coordinates {
                    x: current_pos.x + codel_size,
                    y: current_pos.y,
                })
            {
                not_counted.push(Coordinates {
                    x: current_pos.x + codel_size,
                    y: current_pos.y,
                });
            }
            if current_pos.x - codel_size >= 0 as i32
                && rgb_img[current_pos.y as usize][(current_pos.x - codel_size) as usize] == color
                && !counted.contains(&Coordinates {
                    x: current_pos.x - codel_size,
                    y: current_pos.y,
                })
            {
                not_counted.push(Coordinates {
                    x: current_pos.x - codel_size,
                    y: current_pos.y,
                });
            }
            if current_pos.y + codel_size < rgb_img.len() as i32
                && rgb_img[(current_pos.y + codel_size) as usize][current_pos.x as usize] == color
                && !counted.contains(&Coordinates {
                    x: current_pos.x,
                    y: current_pos.y + codel_size,
                })
            {
                not_counted.push(Coordinates {
                    x: current_pos.x,
                    y: current_pos.y + codel_size,
                });
            }
            if current_pos.y - codel_size >= 0 as i32
                && rgb_img[(current_pos.y - codel_size) as usize][current_pos.x as usize] == color
                && !counted.contains(&Coordinates {
                    x: current_pos.x,
                    y: current_pos.y - codel_size,
                })
            {
                not_counted.push(Coordinates {
                    x: current_pos.x,
                    y: current_pos.y - codel_size,
                });
            }

            current_pos.x += codel_size;
        }
        if not_counted.len() > 0 {
            current_pos = not_counted[0];
            // println!("{:?}", current_pos);
        } else {
            break;
        };
    }
    counted.len() as i32
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
    let mut pos = Coordinates { x: 0, y: 0 };
    let mut block_size = 0;

    let result = get_block_size(rgb_img, pos, 5); // 5 just for lldb debugging
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
    fn get_correct_block_size() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        check_valid_png(&mut file);
        let rgb_img = decode_png(file);

        let result = get_block_size(rgb_img, Coordinates { x: 0, y: 0 }, 5);
        assert_eq!(result, 72);
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
