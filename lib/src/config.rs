use std::path::PathBuf;

pub trait UnitConfig {
    type Result;
    fn parse<P: AsRef<PathBuf>>(path: P) -> Self::Result;
}
