use systemd_parser_macro::{UnitConfig, UnitSection};

#[derive(UnitConfig)]
pub struct Unit {
    pub section_1: SimpleSection,
    pub section_2: AdvancedSection,
}

#[derive(UnitSection)]
pub struct SimpleSection {
    pub field: String,
}

#[derive(UnitSection)]
// sections can be attributed with default to fallback to default when not present
// trait `Default` is required to be implemented
// sections can also have alternative key name
#[unit(default, key = "alt-key")]
pub struct AdvancedSection {
    /// a regular public config field
    pub regular: String,

    /// a private config field
    private: String,

    /// an optional config field
    optional: String,

    /// a vector config field
    vector: Vec<String>,

    /// a nested config field
    nested: NestedConfig,

    /// a config field with custom key name
    #[unit(key = "alt-key")]
    custom_named: String,

    /// a config field with default value
    #[unit(default = "default-value")]
    default_valued: String,
}
