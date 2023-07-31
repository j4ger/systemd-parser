#![allow(non_snake_case)]

use chrono::Duration;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use systemd_parser::prelude::*;

#[derive(UnitConfig)]
struct Unit {
    Sec1: Section,
}

#[derive(UnitSection)]
struct Section {
    Str: String,
    Int: u32,
    Vec: Vec<u32>,
    Dur: Duration,
    Boo: bool,

    #[entry(default = 1919810)]
    Def: u32,

    #[entry(key = "Key")]
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
        b.iter(|| Unit::load_from_string(black_box(SOURCE)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
