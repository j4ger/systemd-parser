use pest::Parser;
use pest_derive::Parser;
use snafu::prelude::*;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[grammar = "unit.pest"]
struct UnitFileParser;

type RuleError = pest::error::Error<Rule>;

#[derive(Debug, Snafu)]
pub enum Error {
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
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn parse<S: AsRef<str>>(input: S) -> Result<HashMap<String, HashMap<String, String>>> {
    let mut parse =
        UnitFileParser::parse(Rule::unit_file, input.as_ref()).context(ParsingSnafu {})?;
    // should never fail since rule unit_file restricts SOI and EOI
    let sectors = parse.next().unwrap().into_inner();

    let mut result = HashMap::new();

    for sector in sectors {
        ensure!(
            sector.as_rule() != Rule::sector,
            SectorSnafu {
                actual: sector.as_rule(),
            }
        );

        let mut inner = sector.into_inner();

        // should not fail since sectors minimum is restricted
        let first_item = inner.next().unwrap();
        // probably also not needed as it would violate grammar test, but if we make the grammar
        // less restrictive, then error messages would be more detailed
        ensure!(
            first_item.as_rule() == Rule::sector_header,
            SectorNameSnafu {
                actual: first_item.as_rule()
            }
        );
        let sector_name = first_item.as_str();

        let mut entries = HashMap::new();

        for entry in inner {
            ensure!(
                entry.as_rule() == Rule::entry,
                EntrySnafu {
                    actual: entry.as_rule()
                }
            );

            let mut entry_inner = entry.into_inner();

            // should not fail as the contents of an entry is restricted
            let key = entry_inner.next().unwrap();
            ensure!(
                key.as_rule() == Rule::key,
                EntryKeySnafu {
                    actual: key.as_rule()
                }
            );
            let key = key.as_str();

            // should not fail as the contents of an entry is restricted
            let values = entry_inner.next().unwrap();
            ensure!(
                values.as_rule() == Rule::value,
                EntryValueSnafu {
                    actual: values.as_rule()
                }
            );
            let value = values.as_str();

            entries.insert(key, value);
        }

        result.insert(sector_name, entries);
    }

    ensure!(!result.is_empty(), NoSectorSnafu {});

    todo!();
}
