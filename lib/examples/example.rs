#![allow(non_snake_case, dead_code)]

use chrono::Duration;
use systemd_parser::prelude::*;

#[derive(UnitConfig, Debug)]
pub struct Unit {
    // sections can be attributed with default to fallback to default when not present
    // trait `Default` is required to be implemented
    // sections can also have alternative key name
    #[section(default, key = "AlternativeKey")]
    pub Section1: SimpleSection,

    pub Section2: AdvancedSection,
}

#[derive(UnitSection, Debug)]
pub struct SimpleSection {
    pub Field: String,
}

impl Default for SimpleSection {
    fn default() -> Self {
        Self {
            Field: "value".to_string(),
        }
    }
}

#[derive(UnitSection, Debug)]
pub struct AdvancedSection {
    /// a regular public config field
    pub Regular: String,

    /// a private config field
    Private: String,

    /// a vector config field
    Vector: Vec<String>,

    /// a config field with values within an enum
    Enum: MyEnum,

    /// a config field with custom key name
    #[entry(key = "AlternativeKey")]
    CustomNamed: String,

    /// a config field with default value
    #[entry(default = "default-value")]
    DefaultValued: String,

    /// a duration config field
    Duration: Duration,
}

#[derive(UnitEntry, Debug)]
enum MyEnum {
    Val1,
    Val2,
}

fn main() {
    let result = Unit::load("examples/example.unit").unwrap();
    println!("{:#?}", result);
}
