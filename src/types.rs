#[derive(Copy, Clone)]
pub enum Direction {
    // maybe as hashmap
    RIGHT,
    DOWN,
    LEFT,
    UP,
}

pub struct ColorInfo {
    pub color: RGB,
    pub size: i32,
}

impl Direction {
    pub fn cords(&self) -> Coordinates {
        match self {
            Direction::RIGHT => Coordinates { x: 1, y: 0 }, //RIGHT
            Direction::DOWN => Coordinates { x: 0, y: 1 },  // DOWN
            Direction::LEFT => Coordinates { x: -1, y: 0 }, // LEFT
            Direction::UP => Coordinates { x: 0, y: -1 },   // UP
        }
    }
    pub fn next(&self) -> Direction {
        match self {
            Direction::RIGHT => Direction::DOWN,
            Direction::DOWN => Direction::LEFT,
            Direction::LEFT => Direction::UP,
            Direction::UP => Direction::RIGHT,
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Coordinates {
    pub x: i32,
    pub y: i32,
}

pub enum CodelChooser {
    LEFT,
    RIGHT,
}

impl CodelChooser {
    pub fn toggle(&self) -> CodelChooser {
        match self {
            CodelChooser::LEFT => CodelChooser::RIGHT,
            CodelChooser::RIGHT => CodelChooser::LEFT,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RGB(pub u8, pub u8, pub u8);
