use snafu::OptionExt;

use crate::{
    config::Result,
    error::{Error, InvalidFilenameSnafu},
};

pub(crate) enum UnitType<'a> {
    Template(&'a str),          // template name
    Instance(&'a str, &'a str), // instance name, template name
    Regular(&'a str),           // unit name
}

pub(crate) fn unit_type<'a>(filename: &'a str) -> Result<UnitType<'a>> {
    let split: Vec<&str> = filename.split("@").collect();
    match split.len() {
        1 => Ok(UnitType::Regular(filename)),
        2 => {
            if split.get(1).unwrap().starts_with('.') {
                Ok(UnitType::Template(split.get(0).unwrap()))
            } else {
                let template_name =
                    split
                        .get(1)
                        .unwrap()
                        .split(".")
                        .nth(0)
                        .context(InvalidFilenameSnafu {
                            filename: filename.to_string(),
                        })?;
                Ok(UnitType::Instance(split.get(0).unwrap(), template_name))
            }
        }
        _ => Err(Error::InvalidFilenameError {
            filename: filename.to_string(),
        }),
    }
}
