#![allow(unused)]
mod tests;
use std::io;
use std::io::Write;

use crate::types::*;
use std::io::Read;

macro_rules! unwrap_or_return {
    ($e:expr) => {
        match $e {
            Some(x) => x,
            None => return,
        }
    };
    ($e:expr, $s:expr) => {
        match $e {
            Some(x) => x,
            None => return $s,
        }
    };
}

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

const CMD: [[for<'r, 's, 't0> fn(i32, &'r mut Vec<i32>, &'s mut CodelChooser, &'t0 mut Direction);
    3]; 6] = [
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
    let _ = unwrap_or_return!(stack.pop());
}
fn add(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());

    stack.push(top + sec_top)
}

fn modulo(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());
    // ignore command if div by 0 (recommended)
    if top == 0 {
        return;
    }

    stack.push(sec_top.rem_euclid(top))
}

fn not(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    if top != 0 {
        stack.push(0)
    } else {
        stack.push(1)
    }
}
fn sub(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());

    stack.push(sec_top - top)
}
fn mult(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());

    stack.push(sec_top * top)
}
fn div(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());
    // ignore command if div by 0 (recommended)
    if top == 0 {
        return;
    }

    stack.push(sec_top / top)
}
fn greater(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    let sec_top = unwrap_or_return!(stack.pop());

    if sec_top > top {
        stack.push(1)
    } else {
        stack.push(0)
    }
}
fn pointer(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    for _ in 0..top {
        *dp = dp.next();
    }
}
fn switch(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    for _ in 0..top.abs() {
        *cc = cc.toggle();
    }
}
fn dup(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    stack.push(top);
    stack.push(top);
}
fn roll(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let rolls = unwrap_or_return!(stack.pop());
    let depth = unwrap_or_return!(stack.pop());
    let len = stack.len();

    if depth <= 0 || stack.len() < depth as usize {
        // ignore command
        stack.push(depth);
        stack.push(rolls);
    } else {
        let rolls = rolls % depth;
        let mut sub = stack.split_off(len - depth as usize);
        if rolls > 0 {
            sub.rotate_right(rolls as usize)
        } else {
            sub.rotate_left((rolls * -1) as usize)
        }
        stack.append(&mut sub)
    }
}

fn in_num(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    print!("> ");
    // forces to print everything everything before input
    io::stdout().flush();

    let mut buffer = String::new();
    let stdin = io::stdin()
        .read_line(&mut buffer)
        .expect("Utf-8 encoded input");

    match buffer.trim().parse::<i32>() {
        Ok(n) => stack.push(n),
        Err(e) => eprintln!("input not a number"),
    }
}
fn in_char(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    print!("> ");
    io::stdout().flush();

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
    let top = unwrap_or_return!(stack.pop());
    print!("{}", top as i32)
}
fn out_char(size: i32, stack: &mut Vec<i32>, cc: &mut CodelChooser, dp: &mut Direction) {
    let top = unwrap_or_return!(stack.pop());
    print!("{}", std::char::from_u32(top as u32).unwrap())
}

pub fn get_color_index(color: RGB) -> Option<Coordinates> {
    for (y, dark_arr) in COLORS.iter().enumerate() {
        for (x, item) in dark_arr.iter().enumerate() {
            if color == *item {
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
    let prev = unwrap_or_return!(get_color_index(prev_color), Coordinates { x: 0, y: 0 });
    let current = unwrap_or_return!(get_color_index(color), Coordinates { x: 0, y: 0 });

    Coordinates {
        x: (current.x - prev.x).rem_euclid(6), // basically pythons () % 6
        y: (current.y - prev.y).rem_euclid(3),
    }
}

pub fn execute(
    stack: &mut Vec<i32>,
    dp: &mut Direction,
    cc: &mut CodelChooser,
    prev: ColorInfo,
    current: &ColorInfo,
) {
    let color_diff = calculate_color_diff(prev.color, current.color);

    CMD[color_diff.x as usize][color_diff.y as usize](prev.size, stack, cc, dp);
}
