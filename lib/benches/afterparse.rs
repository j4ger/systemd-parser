#![allow(non_snake_case)]

use chrono::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unit_parser::prelude::*;

#[derive(UnitConfig, Clone)]
struct Unit {
    Sec1: Option<Section>,
}

#[derive(UnitSection, Clone)]
struct Section {
    #[entry(must)]
    Str: String,

    #[entry(must)]
    Int: u32,

    #[entry(must)]
    Vec: Vec<u32>,

    #[entry(must)]
    Dur: Duration,

    #[entry(must)]
    Boo: bool,

    #[entry(must, default = 1919810)]
    Def: u32,

    #[entry(must, key = "Key")]
    Alt: u32,
}

static SOURCE: &str = r#"
[Section]
Str=Futures made of virtual insanity
Int=42
Vec=7355 608
Dur=6years 1day
Boo=yes
Key=114514
"#;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse string into struct", |b| {
        b.iter(|| Unit::load_from_string(black_box(SOURCE), None))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
