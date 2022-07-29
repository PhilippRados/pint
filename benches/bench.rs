use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pint::navigation::*;
use pint::{decoder::*, types::*};
use std::fs::File;

fn get_block_normal_color_bench(b: &mut Criterion) {
    let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
    check_valid_png(&mut file);
    let rgb_img = black_box(decode_png(file));
    let dp = black_box(Direction::RIGHT);

    b.bench_function("get_block_normal_color", |b| {
        b.iter(|| get_block(&rgb_img, Coordinates { x: 60, y: 0 }, 5, dp))
    });
}

fn get_block_white_color_bench(b: &mut Criterion) {
    let mut file = File::open("tests/fixtures/valentines.png").unwrap();
    check_valid_png(&mut file);
    let rgb_img = black_box(decode_png(file));
    let dp = black_box(Direction::DOWN);

    b.bench_function("get_block_white", |b| {
        b.iter(|| get_block(&rgb_img, Coordinates { x: 1, y: 16 }, 1, dp))
    });
}

criterion_group!(
    benches,
    get_block_normal_color_bench,
    get_block_white_color_bench
);
criterion_main!(benches);
