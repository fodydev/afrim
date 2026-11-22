#![allow(unused_imports, dead_code)]

use afrim_memory::{utils, Cursor};
#[cfg(not(target_arch = "wasm32"))]
use criterion::{criterion_group, criterion_main, Criterion};
use std::{hint::black_box, rc::Rc};

const DATA: &str = include_str!("../data/sample.txt");

#[cfg(not(target_arch = "wasm32"))]
pub fn build_map(c: &mut Criterion) {
    let data = utils::load_data(DATA);

    c.bench_function("load_data", |b| {
        b.iter(|| utils::load_data(black_box(DATA)))
    });
    c.bench_function("build_map", |b| {
        b.iter(|| utils::build_map(black_box(data.clone())))
    });

    // A larger synthetic dataset with long sequential codes.
    let lines: String = (0..20_000)
        .map(|i| format!("{}{i:x}af3uu2  v{i}\n", "abcdefgh".repeat(1 + i % 3)))
        .collect();
    let data = utils::load_data(&lines);
    c.bench_function("build_map_large", |b| {
        b.iter(|| utils::build_map(black_box(data.clone())))
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn cursor(c: &mut Criterion) {
    let memory = Rc::new(utils::build_map(utils::load_data(DATA)));
    let input = "uuaf3afafaff3uu3c_ceduu2iaf2a2a_xx2i22_f".repeat(25);

    c.bench_function("cursor_hit_undo", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(Rc::clone(&memory), 64);
            for character in black_box(input.as_str()).chars() {
                black_box(cursor.hit(character));
            }
            while !cursor.is_empty() {
                black_box(cursor.undo());
            }
        })
    });
}

#[cfg(not(target_arch = "wasm32"))]
criterion_group!(benches, build_map, cursor);

#[cfg(not(target_arch = "wasm32"))]
criterion_main!(benches);

#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("Can't run benchmarks on wasm32");
}
