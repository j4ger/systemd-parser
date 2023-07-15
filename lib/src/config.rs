use std::{collections::HashMap, path::PathBuf};

use crate::error::Result;

/// explicitly derived by using `#[derive(UnitConfig)]`
pub trait UnitConfig: Sized {
    fn parse<P: AsRef<PathBuf>>(path: P) -> Result<Self>;
}

/// explicitly derived by using `#[derive(UnitSection)]`
pub trait UnitSection: Sized {
    fn __parse_section<R: AsRef<str>>(
        source: HashMap<String, HashMap<String, String>>,
        key: R,
    ) -> Result<Self>;
}

/// automatically derived for all supported types
pub trait UnitEntry: Sized {
    fn __parse_entry<R: AsRef<str>>(source: HashMap<String, String>, key: R) -> Result<Self>;
    fn __parse_entry_with_default<S: Into<Self>, R: AsRef<str>>(
        source: HashMap<String, String>,
        key: R,
        default: S,
    ) -> Result<Self>;
}
