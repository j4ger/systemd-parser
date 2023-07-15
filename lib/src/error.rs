use snafu::Snafu;

use crate::parser::Rule;
use std::io;

type RuleError = pest::error::Error<Rule>;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("Failed to read file {}: {}.", path, source))]
    ReadFileError { source: io::Error, path: String },

    #[snafu(display("Failed to parse input: {}.", source))]
    ParsingError { source: RuleError },

    #[snafu(display("Unit file should provide at least one sector."))]
    NoSectorError,

    #[snafu(display("Expecting sector but found {:?}.", actual))]
    SectorError { actual: Rule },

    #[snafu(display("Expecting sector name but found {:?}.", actual))]
    SectorNameError { actual: Rule },

    #[snafu(display("Expecting entry but found {:?}.", actual))]
    EntryError { actual: Rule },

    #[snafu(display("Expecting entry key but found {:?}.", actual))]
    EntryKeyError { actual: Rule },

    #[snafu(display("Expecting entry value but found {:?}.", actual))]
    EntryValueError { actual: Rule },

    #[snafu(display("Missing entry with key {}, which is required.", key))]
    EntryMissingError { key: String },

    #[snafu(display("Missing section with key {}, which is required.", key))]
    SectionMissingError { key: String },

    #[snafu(display("Failed to parse {} as the value of entry with key {}.", value, key))]
    ValueParsingError { key: String, value: String },
}
