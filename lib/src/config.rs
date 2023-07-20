use std::collections::HashMap;

pub type Result<T, E = crate::error::Error> = std::result::Result<T, E>;

/// explicitly derived by using `#[derive(UnitConfig)]`
pub trait UnitConfig: Sized {
    fn parse(__source: &HashMap<String, HashMap<String, String>>) -> Result<Self>;
}

/// explicitly derived by using `#[derive(UnitSection)]`
pub trait UnitSection: Sized {
    fn __parse_section<S: AsRef<str>>(
        __source: &HashMap<String, HashMap<String, String>>,
        __key: S,
    ) -> Result<Option<Self>>;
}

/// automatically derived for all supported types
pub trait UnitEntry: Sized {
    fn __parse_entry<S: AsRef<str>>(
        __source: &HashMap<String, String>,
        __key: S,
    ) -> Result<Option<Self>>;
}
