use crate::types::*;
mod tests;

fn block_dp_corners(dp: &Direction, block: &Vec<Coordinates>) -> (Coordinates, Coordinates) {
    match *dp {
        Direction::RIGHT => {
            let edge: Vec<Coordinates> = block
                .iter()
                .filter(|pos| pos.x == block.iter().max_by_key(|p| p.x).unwrap().x)
                .cloned()
                .collect();
            (
                *edge.iter().min_by_key(|p| p.y).unwrap(),
                *edge.iter().max_by_key(|p| p.y).unwrap(),
            )
        }
        Direction::DOWN => {
            let edge: Vec<Coordinates> = block
                .iter()
                .filter(|pos| pos.y == block.iter().max_by_key(|p| p.y).unwrap().y)
                .cloned()
                .collect();
            (
                *edge.iter().max_by_key(|p| p.x).unwrap(),
                *edge.iter().min_by_key(|p| p.x).unwrap(),
            )
        }
        Direction::LEFT => {
            let edge: Vec<Coordinates> = block
                .iter()
                .filter(|pos| pos.x == block.iter().min_by_key(|p| p.x).unwrap().x)
                .cloned()
                .collect();
            (
                *edge.iter().max_by_key(|p| p.y).unwrap(),
                *edge.iter().min_by_key(|p| p.y).unwrap(),
            )
        }
        Direction::UP => {
            let edge: Vec<Coordinates> = block
                .iter()
                .filter(|pos| pos.y == block.iter().min_by_key(|p| p.y).unwrap().y)
                .cloned()
                .collect();
            (
                *edge.iter().min_by_key(|p| p.x).unwrap(),
                *edge.iter().max_by_key(|p| p.x).unwrap(),
            )
        }
    }
}

fn next_pos(
    dp: &Direction,
    cc: &CodelChooser,
    block: &Vec<Coordinates>,
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

    if !in_range(&new_pos, rgb_img) || is_color(&new_pos, &rgb_img, RGB(0, 0, 0)) {
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
    const CORDS: [Coordinates; 4] = [
        Coordinates { x: 1, y: 0 },  //RIGHT
        Coordinates { x: 0, y: 1 },  // DOWN
        Coordinates { x: -1, y: 0 }, // LEFT
        Coordinates { x: 0, y: -1 }, // UP
    ];

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

pub fn get_block(rgb_img: &Vec<Vec<RGB>>, pos: Coordinates, codel_size: i32) -> Vec<Coordinates> {
    let mut counted: Vec<Coordinates> = Vec::new();
    let mut not_counted: Vec<Coordinates> = Vec::new();
    not_counted.push(Coordinates { ..pos });
    let color = rgb_img[pos.y as usize][pos.x as usize];
    let mut current_pos = pos;

    while not_counted.len() > 0 {
        while in_range(&current_pos, &rgb_img) && is_color(&current_pos, &rgb_img, color) {
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

    let block = get_block(&rgb_img, *pos, codel_size);
    let block_size = get_size(&block);

    // loops until found next color-block
    loop {
        match next_pos(&dp, &cc, &block, codel_size, &rgb_img) {
            Some(new_pos) => {
                cc_toggled = false;
                rotations = 0;
                *pos = new_pos;
                return Some(ColorInfo {
                    color: rgb_img[new_pos.y as usize][new_pos.x as usize],
                    size: block_size,
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
