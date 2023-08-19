#![allow(non_snake_case, dead_code)]

use unit_parser::prelude::*;

#[derive(UnitConfig, Debug, Clone)]
#[unit(suffix = "service")]
struct Unit {
    #[section(must)]
    Section: Section,
}

#[derive(UnitSection, Debug, Clone)]
struct Section {
    #[entry(must)]
    Field1: u32,

    #[entry(must)]
    Field2: u32,

    #[entry(must)]
    Field3: u32,
}

fn main() {
    let result = Unit::load_dir("lib/examples/dropins").unwrap();

    println!("result: {:#?}", result);
}
