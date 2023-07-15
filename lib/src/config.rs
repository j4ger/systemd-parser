use std::{collections::HashMap, path::PathBuf};

use crate::error::Result;

/// explicitly derived by using `#[derive(UnitConfig)]`
pub trait UnitConfig: Sized {
    fn parse<P: AsRef<PathBuf>>(path: P) -> Result<Self>;
}

/// explicitly derived by using `#[derive(UnitSection)]`
pub trait UnitSection: Sized {
    fn __parse_section(source: &HashMap<String, HashMap<String, String>>) -> Result<Option<Self>>;
}

/// automatically derived for all supported types
pub trait UnitEntry: Sized {
    fn __parse_entry(source: &HashMap<String, String>) -> Result<Option<Self>>;
}
