use crate::{
    error::{InvalidDirectorySnafu, ReadDirectorySnafu, ReadEntrySnafu, ReadFileSnafu},
    internal::Error,
    parser::{SectionParser, UnitParser},
};
use snafu::{ensure, ResultExt};
use std::{
    ffi::OsString,
    fs::{read_dir, File},
    io::Read,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
    },
    path::{Path, PathBuf},
    str::FromStr,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait UnitConfig: Sized + Clone {
    const SUFFIX: &'static str;

    fn load_dir<S: AsRef<Path>>(path: S) -> Result<Vec<Self>> {
        let path = path.as_ref();
        ensure!(
            path.is_dir(),
            InvalidDirectorySnafu {
                path: path.to_string_lossy().to_string()
            }
        );

        let mut result = Vec::new();

        for file in read_dir(path).context(ReadDirectorySnafu {
            path: path.to_string_lossy().to_string(),
        })? {
            let file = file.context(ReadEntrySnafu {})?;
            if file.file_type().context(ReadEntrySnafu {})?.is_dir() {
                let inner = Self::load_dir(file.path())?;
                result.extend_from_slice(&inner);
            } else {
                let parse = Self::load(file.path(), None)?;
                result.push(parse);
            }
        }

        Ok(result)
    }

    fn __parse_unit(__source: UnitParser, __from: Option<&Self>) -> Result<Self>;

    fn load_from_string<S: AsRef<str>>(source: S, from: Option<&Self>) -> Result<Self> {
        let parser = crate::parser::UnitParser::new(source.as_ref())?;
        Self::__parse_unit(parser, from)
    }

    fn load<S: AsRef<Path>>(path: S, from: Option<&Self>) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        Self::load_from_string(content, from)
    }
}

pub trait UnitSection: Sized + Clone {
    fn __parse_section(__source: SectionParser, __from: Option<Self>) -> Result<Option<Self>>;
}

pub trait UnitEntry: Sized + Clone {
    type Error;
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error>;
}

pub trait EntryInner {}

macro_rules! impl_for_types {
    ($typ:ty) => {
        impl UnitEntry for $typ {
            type Error = <$typ as FromStr>::Err;
            fn parse_from_str<S: AsRef<str>>(
                input: S,
            ) -> std::result::Result<Self, Self::Error> {
                Self::from_str(input.as_ref())
            }
        }

        impl EntryInner for $typ {}
    };
    ($x:ty, $($y:ty),+) => {
        impl_for_types!($x);
        impl_for_types!($($y),+);
    };
}

impl_for_types!(
    IpAddr,
    SocketAddr,
    char,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    OsString,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddrV4,
    SocketAddrV6,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
    PathBuf,
    String
);

impl UnitEntry for bool {
    type Error = ();
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
        match input.as_ref() {
            "1" | "yes" | "true" | "on" => Ok(true),
            "0" | "no" | "false" | "off" => Ok(false),
            _ => Err(()),
        }
    }
}

impl<T: UnitEntry + EntryInner> UnitEntry for Vec<T> {
    type Error = <T as UnitEntry>::Error;
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error> {
        let mut result = Vec::new();
        for value in input.as_ref().split_ascii_whitespace() {
            result.push(T::parse_from_str(value)?);
        }
        Ok(result)
    }
}
