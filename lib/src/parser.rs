use std::{path::Path, rc::Rc};

use crate::{
    config::Result,
    error::*,
    specifiers::{resolve, SpecifierContext},
};
use pest::{iterators::Pairs, Parser};
use pest_derive::Parser;
use snafu::ResultExt;

#[derive(Parser, Debug)]
#[grammar = "unit.pest"]
pub struct UnitFileParser;

pub struct UnitParser<'a> {
    context: Rc<SpecifierContext>,
    filename: &'a str,
    path: &'a Path,
    inner: Pairs<'a, Rule>,
}

impl<'a> UnitParser<'a> {
    pub(crate) fn new(
        input: &'a str,
        context: Rc<SpecifierContext>,
        filename: &'a str,
        path: &'a Path,
    ) -> Result<Self> {
        let mut parse =
            UnitFileParser::parse(Rule::unit_file, input.as_ref()).context(ParsingSnafu {})?;
        // should never fail since rule unit_file restricts SOI and EOI
        let sections = parse.next().unwrap().into_inner();
        Ok(Self {
            inner: sections,
            filename,
            path,
            context,
        })
    }
}

impl<'a> Iterator for UnitParser<'a> {
    type Item = Result<SectionParser<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next().unwrap();
        if item.as_rule() == Rule::EOI {
            return None;
        }

        if item.as_rule() != Rule::section {
            return Some(Err(Error::SectionError {
                actual: item.as_rule(),
            }));
        }

        let mut inner = item.into_inner();

        let first_item = inner.next().unwrap();

        // probably also not needed as it would have already violated grammar test, but if we make the grammar
        // less restrictive, then error messages would be more detailed
        if first_item.as_rule() != Rule::section_header {
            return Some(Err(Error::SectionNameError {
                actual: first_item.as_rule(),
            }));
        }

        let section_name = first_item.as_str();

        let context = Rc::clone(&self.context);

        Some(Ok(SectionParser {
            name: section_name,
            inner,
            context,
            path: self.path,
            filename: self.filename,
        }))
    }
}

pub struct SectionParser<'a> {
    pub name: &'a str,
    inner: Pairs<'a, Rule>,
    filename: &'a str,
    path: &'a Path,
    context: Rc<SpecifierContext>,
}

impl<'a> Iterator for SectionParser<'a> {
    type Item = Result<(&'a str, String)>;
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.inner.next();
        if let Some(entry) = entry {
            if entry.as_rule() != Rule::entry {
                return Some(Err(Error::EntryError {
                    actual: entry.as_rule(),
                }));
            }

            let mut entry_inner = entry.into_inner();

            // should not fail as the contents of an entry is restricted
            let key = entry_inner.next().unwrap();
            if key.as_rule() != Rule::key {
                return Some(Err(Error::EntryKeyError {
                    actual: key.as_rule(),
                }));
            }
            let key = key.as_str();

            // should not fail as the contents of an entry is restricted
            let values = entry_inner.next().unwrap();
            if values.as_rule() != Rule::value {
                return Some(Err(Error::EntryValueError {
                    actual: values.as_rule(),
                }));
            }

            let mut value = String::new();
            for item in values.into_inner() {
                if item.as_rule() == Rule::value_block {
                    value.push_str(item.as_str());
                } else {
                    value.push_str(
                        resolve(
                            item.as_str().chars().nth(0).unwrap(),
                            self.context.as_ref(),
                            self.filename,
                            self.path,
                        )
                        .unwrap_or("".to_string())
                        .as_str(),
                    );
                }
            }

            return Some(Ok((key, value)));
        } else {
            return None;
        }
    }
}
