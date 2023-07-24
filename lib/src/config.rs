use snafu::ResultExt;

use crate::{error::ReadFileSnafu, internal::Error};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs::File,
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

/// explicitly derived by using `#[derive(UnitConfig)]`
pub trait UnitConfig: Sized {
    fn parse(__source: &HashMap<String, HashMap<String, String>>) -> Result<Self>;
    fn load<S: AsRef<str>>(__path: S) -> Result<Self> {
        let path = Path::new(__path.as_ref());
        let mut file = File::open(path).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let map = crate::parser::parse(content)?;
        Self::parse(&map)
    }
}

/// explicitly derived by using `#[derive(UnitSection)]`
pub trait UnitSection: Sized {
    fn __parse_section<S: AsRef<str>>(
        __source: &HashMap<String, HashMap<String, String>>,
        __key: S,
    ) -> Result<Option<Self>>;
}

/// automatically derived for all supported types
pub trait UnitEntry: Sized {
    type Error;
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error>;
    fn __parse_entry<S: AsRef<str>>(
        __source: &HashMap<String, String>,
        __key: S,
    ) -> Result<Option<Self>> {
        let key = __key.as_ref();
        match __source.get(key) {
            None => Ok(None),
            Some(value) => {
                let value = Self::parse_from_str(value).map_err(|_| Error::ValueParsingError {
                    key: key.to_string(),
                    value: value.to_string(),
                })?;
                Ok(Some(value))
            }
        }
    }
}

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
    };
    ($x:ty, $($y:ty),+) => {
        impl_for_types!($x);
        impl_for_types!($($y),+);
    };
}

macro_rules! impl_for_vec_types {
    ($typ:ty) => {
        impl UnitEntry for Vec<$typ> {
            type Error = <$typ as FromStr>::Err;
            fn parse_from_str<S: AsRef<str>>(
                input: S,
            ) -> std::result::Result<Self, Self::Error> {
                let input = input.as_ref().split_ascii_whitespace();
                let mut result = Vec::new();
                for value in input {
                    let member = <$typ>::from_str(value)?;
                    result.push(member);
                }
                Ok(result)
            }
        }
    };
    ($x:ty, $($y:ty),+) => {
        impl_for_vec_types!($x);
        impl_for_vec_types!($($y),+);
    };
}

impl_for_types!(
    IpAddr,
    SocketAddr,
    bool,
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

impl_for_vec_types!(
    IpAddr,
    SocketAddr,
    bool,
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
