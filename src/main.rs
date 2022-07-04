#![allow(unused)]
use std::fs::File;
use std::io;
use std::io::Read;

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
const CMD: [[for<'r, 's, 't0> fn(
    i32,
    &'r mut Vec<i32>,
    &'s mut types::CodelChooser,
    &'t0 mut types::Direction,
); 3]; 6] = [
    [none, push, pop],
    [add, sub, mult],
    [div, modulo, not],
    [greater, pointer, switch],
    [dup, roll, in_num],
    [in_char, out_num, out_char],
];

fn none(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {}
fn push(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    stack.push(size)
}
fn pop(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let _ = stack.pop().expect("Cannot pop from empty stack");
    ()
}
fn add(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot add: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();

    stack.push(top + sec_top)
}

fn modulo(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot mod: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();
    // ignore command if div by 0 (recommended)
    if top == 0 {
        return;
    }

    stack.push(sec_top.rem_euclid(top))
}

fn not(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cannot pop from empty stack");
    if top != 0 {
        stack.push(0)
    } else {
        stack.push(1)
    }
}
fn sub(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot sub: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();

    stack.push(sec_top - top)
}
fn mult(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot mult: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();

    stack.push(sec_top * top)
}
fn div(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot mod: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();
    // ignore command if div by 0 (recommended)
    if top == 0 {
        return;
    }

    stack.push(sec_top / top)
}
fn greater(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot mod: {:?}",
        stack
    );
    let top = stack.pop().unwrap();
    let sec_top = stack.pop().unwrap();

    if sec_top > top {
        stack.push(1)
    } else {
        stack.push(0)
    }
}
fn pointer(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cant pop from empty stack");
    for _ in 0..top {
        dp.next();
    }
}
fn switch(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cant pop from empty stack");
    for _ in 0..top.abs() {
        cc.toggle();
    }
}
fn dup(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cant pop from empty stack");

    stack.push(top);
    stack.push(top);
}
fn roll(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    assert!(
        stack.len() >= 2,
        "Stack holds less than 2 elements cannot roll: {:?}",
        stack
    );
    let rolls = stack.pop().unwrap();
    let depth = stack.pop().unwrap();
    let len = stack.len();

    if depth <= 0 || stack.len() < depth as usize {
        // ignore command
        stack.push(depth);
        stack.push(rolls);
    } else {
        let mut sub = stack.split_off(len - depth as usize);
        if rolls > 0 {
            sub.rotate_right(rolls as usize)
        } else {
            sub.rotate_left(rolls.abs() as usize)
        }
        stack.append(&mut sub)
    }
}

fn in_num(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let mut buffer = String::new();
    let mut stdin = io::stdin()
        .read_line(&mut buffer)
        .expect("Utf-8 encoded input");

    match buffer.parse::<i32>() {
        Ok(n) => stack.push(n),
        Err(e) => panic!("Input not a number"),
    }
}
fn in_char(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let mut buffer = String::new();
    let mut byte_unicode = io::stdin()
        .bytes()
        .next()
        .and_then(|result| result.ok())
        .map(|byte| byte as i32)
        .expect("Utf-8 encoded input");

    stack.push(byte_unicode)
}

fn out_num(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cannot pop from empty stack");
    print!("{}", top as i32)
}
fn out_char(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = stack.pop().expect("Cannot pop from empty stack");
    print!("{}", std::char::from_u32(top as u32).unwrap())
}

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

fn execute(
    stack: &mut Vec<i32>,
    dp: &mut Direction,
    cc: &mut CodelChooser,
    prev: ColorInfo,
    current: &ColorInfo,
) {
    let color_diff = calculate_color_diff(prev.color, current.color);

    CMD[color_diff.x as usize][color_diff.y as usize](current.size, stack, cc, dp);
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

    let mut stack = Vec::new();
    let mut current_color = ColorInfo {
        color: rgb_img[pos.y as usize][pos.x as usize],
        size: get_size(&get_block(&rgb_img, pos, codel_size)),
    };
    loop {
        let mut prev_color = current_color;
        current_color = match next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc) {
            Some(new_color) => new_color,
            None => break,
        };
        execute(&mut stack, &mut dp, &mut cc, prev_color, &current_color);
    }
    println!("");
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
    #[test]
    fn roll_test1() {
        let mut stack = vec![12, 3, 102, 33, 7, 4, 2];
        let mut dp = Direction::UP;
        let mut cc = CodelChooser::LEFT;

        roll(3, &mut stack, &mut cc, &mut dp);
        assert_eq!(stack, [12, 33, 7, 3, 102]);
    }
    #[test]
    fn roll_test2() {
        let mut stack = vec![1, 2, 3, 3, 1];
        let mut dp = Direction::UP;
        let mut cc = CodelChooser::LEFT;

        roll(3, &mut stack, &mut cc, &mut dp);
        assert_eq!(stack, [3, 1, 2]);
    }
}
