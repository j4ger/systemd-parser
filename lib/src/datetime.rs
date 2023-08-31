use crate::config::UnitEntry;
use chrono::Utc;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "datetime.pest"]
pub(crate) struct DatetimeParser {}

impl UnitEntry for chrono::DateTime<Utc> {
    type Error = ();
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
        let segments: Vec<&str> = input.as_ref().split_ascii_whitespace().collect();
        if segments.is_empty() {
            return Ok(Utc::now());
        }
        todo!();
    }
}
