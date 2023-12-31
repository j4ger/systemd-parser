use crate::{
    error::ReadFileSnafu,
    internal::Error,
    parser::{SectionParser, UnitParser},
    template::{unit_type, UnitType},
};
use snafu::ResultExt;
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
    rc::Rc,
    str::FromStr,
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait UnitConfig: Sized {
    const SUFFIX: &'static str;
    fn __parse_unit(__source: UnitParser) -> Result<Self>;
    fn __patch_unit(__source: UnitParser, __from: &mut Self) -> Result<()>;

    fn __load<S: AsRef<Path>>(
        path: S,
        paths: Rc<Vec<PathBuf>>,
        filename: &str,
        root: bool,
    ) -> Result<Self> {
        let path = path.as_ref();
        let mut file = File::open(path).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let parser = crate::parser::UnitParser::new(content.as_ref(), paths, root, filename, path)?;
        Self::__parse_unit(parser)
    }

    fn __patch<S: AsRef<Path>>(
        path: S,
        paths: Rc<Vec<PathBuf>>,
        filename: &str,
        from: &mut Self,
        root: bool,
    ) -> Result<()> {
        let path = path.as_ref();
        let mut file = File::open(path).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let parser = crate::parser::UnitParser::new(content.as_ref(), paths, root, filename, path)?;
        Self::__patch_unit(parser, from)
    }

    fn load<S: AsRef<Path>>(path: S, root: bool) -> Result<Self> {
        let path = path.as_ref();
        let empty_vec: Vec<PathBuf> = Vec::new();
        let paths = Rc::new(empty_vec);
        Self::__load(
            path,
            paths,
            path.file_name()
                .map_or("".to_string(), |x| x.to_string_lossy().to_string())
                .as_str(),
            root,
        )
    }

    fn load_named<S: AsRef<str>, P: AsRef<Path>>(
        paths: Vec<P>,
        name: S,
        root: bool,
    ) -> Result<Self> {
        // return when first one is found?
        let paths: Vec<PathBuf> = paths.iter().map(|x| x.as_ref().to_path_buf()).collect();
        let paths_rc = Rc::new(paths);
        let name = name.as_ref();
        let fullname = if name.ends_with(Self::SUFFIX) {
            name.to_string()
        } else {
            format!("{}.{}", name, Self::SUFFIX)
        };
        let actual_file_name = match unit_type(fullname.as_str())? {
            UnitType::Template(_) => {
                return Err(Error::LoadTemplateError {
                    name: fullname.to_owned(),
                });
            }
            UnitType::Instance(_, template_filename) => template_filename,
            UnitType::Regular(_) => fullname.to_owned(),
        };
        let mut result = None;

        // load itself
        let paths = Rc::clone(&paths_rc);
        for dir in (*paths).iter() {
            let mut path = dir.to_owned();
            path.push(actual_file_name.as_str());
            if let Ok(res) = Self::__load(path, Rc::clone(&paths_rc), fullname.as_str(), root) {
                result = Some(res);
                break;
            }
        }

        let mut result = if let Some(result) = result {
            result
        } else {
            return Err(Error::NoUnitFoundError {
                name: name.to_string(),
            });
        };

        // load drop-ins
        let mut dropin_dir_names = vec![
            format!("{}.d", Self::SUFFIX),
            format!("{}.d", fullname.as_str()),
        ];
        let segments: Vec<&str> = fullname.split('-').collect();
        for i in (1..segments.len()).rev() {
            let segmented = segments[0..i].join("-");
            let dir_name = format!("{}-.{}.d", segmented, Self::SUFFIX);
            dropin_dir_names.push(dir_name);
        }

        for dir_name in dropin_dir_names.iter() {
            for dir in (*paths).iter() {
                let mut path = dir.to_owned();
                path.push(dir_name.as_str());
                if path.is_dir() {
                    if let Ok(dir_entries) = read_dir(&path) {
                        for item in dir_entries {
                            if let Ok(entry) = item {
                                if let Ok(meta) = entry.metadata() {
                                    if meta.is_file()
                                        && entry.path().extension().is_some_and(|x| x == "conf")
                                    {
                                        let paths = Rc::clone(&paths_rc);
                                        if let Err(err) = Self::__patch(
                                            entry.path(),
                                            paths,
                                            fullname.as_str(),
                                            &mut result,
                                            root,
                                        ) {
                                            log::warn!("Failed to patch unit {}: {})", name, err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

pub trait UnitSection: Sized {
    fn __parse_section(__source: SectionParser) -> Result<Option<Self>>;
    fn __patch_section(__source: SectionParser, __from: &mut Self) -> Result<()>;
}

pub trait UnitEntry: Sized {
    type Error;
    fn parse_from_str<S: AsRef<str>>(input: S) -> std::result::Result<Self, Self::Error>;
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
