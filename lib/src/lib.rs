mod config;
mod error;
mod parser;

pub use config::{Result, UnitConfig, UnitEntry, UnitSection};
pub use error::Error;
pub use parser::{parse, parse_file};
pub use systemd_parser_macro::{UnitConfig, UnitSection};
