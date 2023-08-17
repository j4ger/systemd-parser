#![allow(non_snake_case, dead_code)]

use unit_parser::prelude::*;

#[derive(UnitConfig, Debug, Clone)]
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
}

fn main() {
    let base = Unit::load("examples/base.unit", None).unwrap();
    let patch = Unit::load("examples/patch.unit", Some(&base)).unwrap();

    println!("base: {:#?}", base);
    println!("patch: {:#?}", patch);
}
