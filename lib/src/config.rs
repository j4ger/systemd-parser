use crate::internal::Error;
use std::{collections::HashMap, str::FromStr};

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

impl<T: FromStr> UnitEntry for T {
    fn __parse_entry<S: AsRef<str>>(
        __source: &HashMap<String, String>,
        __key: S,
    ) -> Result<Option<Self>> {
        let key = __key.as_ref();
        match __source.get(key) {
            None => Ok(None),
            Some(value) => {
                let value = T::from_str(value).map_err(|_| Error::ValueParsingError {
                    key: key.to_string(),
                    value: value.to_string(),
                })?;
                Ok(Some(value))
            }
        }
    }
}

pub struct SimpleSection {
    pub field: String,
}

impl crate::internal::UnitSection for SimpleSection {
    fn __parse_section<S: AsRef<str>>(
        __source: &std::collections::HashMap<String, std::collections::HashMap<String, String>>,
        __key: S,
    ) -> crate::internal::Result<Option<Self>> {
        let __source = match __source.get(__key.as_ref()) {
            Some(__inner) => __inner,
            None => {
                return Ok(None);
            }
        };
        let field = String::__parse_entry(__source, "field")?.ok_or(
            crate::internal::Error::EntryMissingError {
                key: "field".to_string(),
            },
        )?;
        Ok(Some(Self { field }))
    }
}
