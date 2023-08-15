use chrono::Utc;

use crate::config::UnitEntry;

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
