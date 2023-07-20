use std::{fs::File, io::Read};

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

    // /// a vector config field
    // vector: Vec<String>,

    // /// a config field with values within an enum
    // enum_field: MyEnum,
    /// a config field with custom key name
    #[entry(key = "AlternativeKey")]
    CustomNamed: String,

    /// a config field with default value
    #[entry(default = "default-value")]
    DefaultValued: String,
}

// #[derive(UnitEntry)]
// enum MyEnum {
//     Val1,
//     Val2,
// }

fn main() {
    let mut file = File::open("./examples/example.unit").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let hashmap = systemd_parser::internal::parse(content).unwrap();
    let result = Unit::parse(&hashmap).unwrap();
    println!("{:?}", result);
}
