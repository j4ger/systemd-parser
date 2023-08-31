#![allow(non_snake_case, dead_code)]

use unit_parser::prelude::*;

#[derive(UnitConfig, Debug, Clone)]
#[unit(suffix = "unit")]
struct Unit {
    #[section(must)]
    Section: Section,
}

#[derive(UnitSection, Debug, Clone)]
struct Section {
    #[entry(must, multiple, subdir = "wants")]
    Wants: Vec<String>,
}

fn main() {
    let result = Unit::load_named(vec!["lib/examples/subdir"], "subdir.unit", false).unwrap();

    println!("result: {:#?}", result);
}
