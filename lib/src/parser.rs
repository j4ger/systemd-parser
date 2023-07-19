use crate::{config::Result, error::*};
use pest::Parser;
use pest_derive::Parser;
use snafu::{ensure, ResultExt};
use std::{collections::HashMap, fs::File, io::Read, path::Path};

#[derive(Parser, Debug)]
#[grammar = "unit.pest"]
pub struct UnitFileParser;

type Output = Result<HashMap<String, HashMap<String, String>>>;

pub fn parse<S: AsRef<str>>(input: S) -> Output {
    let mut parse =
        UnitFileParser::parse(Rule::unit_file, input.as_ref()).context(ParsingSnafu {})?;
    // should never fail since rule unit_file restricts SOI and EOI
    let sectors = parse.next().unwrap().into_inner();

    let mut result = HashMap::new();

    for sector in sectors {
        if sector.as_rule() == Rule::EOI {
            break;
        }

        ensure!(
            sector.as_rule() == Rule::sector,
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
        let sector_name = first_item.as_str().to_string();

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
            let key = key.as_str().to_string();

            // should not fail as the contents of an entry is restricted
            let values = entry_inner.next().unwrap();
            ensure!(
                values.as_rule() == Rule::value,
                EntryValueSnafu {
                    actual: values.as_rule()
                }
            );
            let mut value = String::new();
            for line in values.into_inner() {
                let partial = line.as_str().trim_end_matches("\\\n");
                value.push_str(partial);
            }

            entries.insert(key, value);
        }

        result.insert(sector_name, entries);
    }

    ensure!(!result.is_empty(), NoSectorSnafu {});

    Ok(result)
}

pub fn parse_file<S: AsRef<Path>>(input_file: S) -> Output {
    let path = input_file.as_ref();
    let mut content = String::new();
    let mut file = File::open(path).context(ReadFileSnafu {
        path: path.to_string_lossy().to_string(),
    })?;
    file.read_to_string(&mut content).context(ReadFileSnafu {
        path: path.to_string_lossy().to_string(),
    })?;
    parse(content)
}
