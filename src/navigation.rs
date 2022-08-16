use crate::interpreter::get_color_index;
use crate::types::*;
use core::slice::Iter;
mod tests;

fn get_x(p: &&Coordinates) -> i32 {
    p.x
}
fn get_y(p: &&Coordinates) -> i32 {
    p.y
}

fn get_min(iter: Iter<'_, Coordinates>, get_field: fn(&&Coordinates) -> i32) -> i32 {
    get_field(&iter.min_by_key(get_field).unwrap())
}
fn get_max(iter: Iter<'_, Coordinates>, get_field: fn(&&Coordinates) -> i32) -> i32 {
    get_field(&iter.max_by_key(get_field).unwrap())
}
type CoordGetter = fn(&&Coordinates) -> i32;

fn block_dp_corners(dp: &Direction, block: &[Coordinates]) -> (Coordinates, Coordinates) {
    let (get_direction_field, get_width_field): (CoordGetter, CoordGetter) = match *dp {
        Direction::RIGHT | Direction::LEFT => (get_x, get_y),
        Direction::DOWN | Direction::UP => (get_y, get_x),
    };
    let min_or_max = match *dp {
        Direction::RIGHT | Direction::DOWN => get_max,
        Direction::LEFT | Direction::UP => get_min,
    };
    let direction_limit = min_or_max(block.iter(), get_direction_field);

    let edge: Vec<Coordinates> = block
        .iter()
        .filter(|pos| get_direction_field(pos) == direction_limit)
        .cloned()
        .collect();

    let min = *edge.iter().min_by_key(get_width_field).unwrap();
    let max = *edge.iter().max_by_key(get_width_field).unwrap();

    match *dp {
        Direction::LEFT | Direction::DOWN => (max, min),
        Direction::RIGHT | Direction::UP => (min, max),
    }
}

fn next_pos(
    dp: &Direction,
    cc: &CodelChooser,
    block: &[Coordinates],
    codel_size: i32,
    rgb_img: &Vec<Vec<RGB>>,
) -> Option<Coordinates> {
    let block_corners = block_dp_corners(dp, block);

    let new_pos = match cc {
        CodelChooser::LEFT => Coordinates {
            x: block_corners.0.x + dp.cords().x * codel_size,
            y: block_corners.0.y + dp.cords().y * codel_size,
        },
        CodelChooser::RIGHT => Coordinates {
            x: block_corners.1.x + dp.cords().x * codel_size,
            y: block_corners.1.y + dp.cords().y * codel_size,
        },
    };

    if !in_range(&new_pos, rgb_img) || is_color(&new_pos, rgb_img, RGB(0, 0, 0)) {
        None
    } else {
        Some(new_pos)
    }
}

fn remove_all<T: Eq>(arr: &mut Vec<T>, element: &T) {
    arr.retain(|e| e != element);
}

fn in_range(new_pos: &Coordinates, rgb_img: &Vec<Vec<RGB>>) -> bool {
    let width = rgb_img[0].len() as i32;
    let height = rgb_img.len() as i32;
    let x_pos = new_pos.x;
    let y_pos = new_pos.y;

    x_pos < width && x_pos >= 0 && y_pos < height && y_pos >= 0
}

fn is_color(new_pos: &Coordinates, rgb_img: &[Vec<RGB>], color: RGB) -> bool {
    rgb_img[new_pos.y as usize][new_pos.x as usize] == color
}

const CORDS: [Coordinates; 4] = [
    Coordinates { x: 1, y: 0 },  //RIGHT
    Coordinates { x: 0, y: 1 },  // DOWN
    Coordinates { x: -1, y: 0 }, // LEFT
    Coordinates { x: 0, y: -1 }, // UP
];
fn check_adjacent_codels(
    current_pos: Coordinates,
    codel_size: i32,
    rgb_img: &Vec<Vec<RGB>>,
    counted: &mut [Coordinates],
    not_counted: &mut Vec<Coordinates>,
    color: RGB,
) {
    for direction in CORDS {
        let new_pos = Coordinates {
            x: current_pos.x + (direction.x * codel_size),
            y: current_pos.y + (direction.y * codel_size),
        };

        if in_range(&new_pos, rgb_img)
            && is_color(&new_pos, rgb_img, color)
            && !counted.contains(&new_pos)
        {
            not_counted.push(new_pos);
        }
    }
}

fn get_last_codel_in_dir(
    current_pos: &mut Coordinates,
    rgb_img: &Vec<Vec<RGB>>,
    color: RGB,
    dp: Direction,
) -> Coordinates {
    let mut result = Vec::new();
    while in_range(current_pos, rgb_img) && is_color(current_pos, rgb_img, color) {
        result.push(Coordinates { ..*current_pos });
        match dp {
            Direction::RIGHT => current_pos.x += 1,
            Direction::DOWN => current_pos.y += 1,
            Direction::LEFT => current_pos.x -= 1,
            Direction::UP => current_pos.y -= 1,
        }
    }
    result.pop().unwrap()
}

pub fn get_block(
    rgb_img: &Vec<Vec<RGB>>,
    pos: Coordinates,
    codel_size: i32,
    dp: Direction,
) -> Vec<Coordinates> {
    let mut counted: Vec<Coordinates> = Vec::new();
    let color = rgb_img[pos.y as usize][pos.x as usize];
    let mut current_pos = pos;

    if get_color_index(rgb_img[pos.y as usize][pos.x as usize]) == None {
        counted.push(get_last_codel_in_dir(&mut current_pos, rgb_img, color, dp));
        return counted;
    }

    let mut not_counted: Vec<Coordinates> = Vec::new();
    not_counted.push(Coordinates { ..pos });

    while !not_counted.is_empty() {
        while in_range(&current_pos, rgb_img) && is_color(&current_pos, rgb_img, color) {
            if not_counted.contains(&current_pos) {
                // remove from not_counted add to counted
                remove_all::<Coordinates>(&mut not_counted, &current_pos);
                counted.push(current_pos);
            }
            // mark adjacent codels as not_counted
            check_adjacent_codels(
                current_pos,
                codel_size,
                rgb_img,
                &mut counted,
                &mut not_counted,
                color,
            );

            current_pos.x += codel_size;
        }
        if !not_counted.is_empty() {
            current_pos = not_counted[0];
        } else {
            break;
        };
    }
    counted
}

pub fn get_size(block: &Vec<Coordinates>) -> i32 {
    block.len() as i32
}

pub fn next_color(
    rgb_img: &Vec<Vec<RGB>>,
    pos: &mut Coordinates,
    codel_size: i32,
    dp: &mut Direction,
    cc: &mut CodelChooser,
) -> Option<ColorInfo> {
    let mut cc_toggled = false;
    let mut rotations = 0;

    let mut block = get_block(rgb_img, *pos, codel_size, *dp);
    // loops until found next color-block
    loop {
        match next_pos(dp, cc, &block, codel_size, rgb_img) {
            Some(new_pos) => {
                *pos = new_pos;
                block = get_block(rgb_img, *pos, codel_size, *dp);
                return Some(ColorInfo {
                    color: rgb_img[pos.y as usize][pos.x as usize],
                    size: get_size(&block),
                });
            }
            None => {
                if rotations >= 4 {
                    return None;
                } else if cc_toggled {
                    *dp = dp.next();
                    cc_toggled = false;
                    rotations += 1;
                } else {
                    *cc = cc.toggle();
                    cc_toggled = true;
                }
            }
        };
    }
}
