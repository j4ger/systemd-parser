use crate::{
    error::ReadFileSnafu,
    internal::Error,
    parser::{SectionParser, UnitParser},
    specifiers::SpecifierContext,
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

pub trait UnitConfig: Sized + Clone {
    const SUFFIX: &'static str;

    // fn load_dir<S: AsRef<Path>>(paths: Vec<S>) -> Result<Vec<(String, Self)>> {
    //     let mut templates = HashMap::new();
    //     let mut instances = HashMap::new();
    //     let mut dropins = HashMap::new();
    //     let mut results = Vec::new();

    //     for path in paths {
    //         Self::load_dir_sub(
    //             path,
    //             &mut templates,
    //             &mut instances,
    //             &mut dropins,
    //             &mut results,
    //         )?;
    //     }

    //     for (template_name, instance_names) in instances.iter() {
    //         match templates.get(template_name) {
    //             Some(template) => {
    //                 for instance_name in instance_names {
    //                     // advanced specifiers patching
    //                     let patched = template.replace("%i", instance_name);
    //                     let parse = Self::load_from_string(patched, None)?;
    //                     results.push((
    //                         format!("{instance_name}@{template_name}.{}", Self::SUFFIX),
    //                         parse,
    //                     ));
    //                 }
    //             }
    //             None => {
    //                 log::warn!("Template {} is not found.", template_name);
    //             }
    //         }
    //     }

    //     for result in results.iter_mut() {
    //         let segments: Vec<&str> = result.0.split("-").collect();
    //         if let Some(drop_in_vec) = dropins.get_mut(format!("{}.d", Self::SUFFIX).as_str()) {
    //             drop_in_vec.sort_unstable_by(|x, y| x.0.cmp(&y.1));
    //             let mut res = result.1.clone();
    //             for drop_in in drop_in_vec {
    //                 res = Self::load_from_string(drop_in.1.to_owned(), Some(&res))?;
    //             }
    //             result.1 = res;
    //         }
    //         if let Some(drop_in_vec) = dropins.get_mut(result.0.as_str()) {
    //             drop_in_vec.sort_unstable_by(|x, y| x.0.cmp(&y.1));
    //             let mut res = result.1.clone();
    //             for drop_in in drop_in_vec {
    //                 res = Self::load_from_string(drop_in.1.to_owned(), Some(&res))?;
    //             }
    //             result.1 = res;
    //         }
    //         for i in (1..segments.len()).rev() {
    //             let segmented = segments[0..i].join("-");

    //             let key = format!("{}-.{}", segmented, Self::SUFFIX);
    //             if let Some(drop_in_vec) = dropins.get_mut(key.as_str()) {
    //                 drop_in_vec.sort_unstable_by(|x, y| x.0.cmp(&y.1));
    //                 let mut res = result.1.clone();
    //                 for drop_in in drop_in_vec {
    //                     res = Self::load_from_string(drop_in.1.to_owned(), Some(&res))?;
    //                 }
    //                 result.1 = res;
    //             }
    //         }
    //     }

    //     Ok(results)
    // }

    // fn load_dir_sub<S: AsRef<Path>>(
    //     path: S,
    //     templates: &mut HashMap<String, String>,
    //     instances: &mut HashMap<String, Vec<String>>,
    //     dropins: &mut HashMap<String, Vec<(String, String)>>,
    //     results: &mut Vec<(String, Self)>,
    // ) -> Result<()> {
    //     let path = path.as_ref();
    //     let path_str = path.to_string_lossy().to_string();
    //     ensure!(
    //         path.is_dir(),
    //         InvalidDirectorySnafu {
    //             path: path_str.to_owned()
    //         }
    //     );
    //     let dir_name = path
    //         .file_name()
    //         .unwrap()
    //         .to_str()
    //         .context(FilenameUnreadableSnafu {
    //             path: path_str.to_owned(),
    //         })?;

    //     for file in read_dir(path).context(ReadDirectorySnafu {
    //         path: path_str.to_owned(),
    //     })? {
    //         let file = file.context(ReadEntrySnafu {})?;
    //         let filename = file.file_name();
    //         let filename = filename
    //             .to_str()
    //             .context(FilenameUnreadableSnafu {
    //                 path: path_str.to_owned(),
    //             })?
    //             .to_string();
    //         if dir_name.ends_with(".d") {
    //             if filename.ends_with(".conf") {
    //                 let unit_name = dir_name.trim_end_matches(".d");
    //                 let mut handle = File::open(file.path()).context(ReadFileSnafu {
    //                     path: path_str.to_owned(),
    //                 })?;
    //                 let mut content = String::new();
    //                 handle.read_to_string(&mut content).context(ReadFileSnafu {
    //                     path: path_str.to_owned(),
    //                 })?;
    //                 match dropins.get_mut(unit_name) {
    //                     Some(current) => {
    //                         current.push((filename, content));
    //                     }
    //                     None => {
    //                         dropins.insert(unit_name.to_string(), vec![(filename, content)]);
    //                     }
    //                 }
    //             }
    //         } else if file.file_type().context(ReadEntrySnafu {})?.is_dir() {
    //             Self::load_dir_sub(file.path(), templates, instances, dropins, results)?;
    //         } else {
    //             if Self::SUFFIX != "" {
    //                 if let Some(extension) = file.path().extension() {
    //                     if let Some(extension) = extension.to_str() {
    //                         if extension != Self::SUFFIX {
    //                             continue;
    //                         }
    //                     } else {
    //                         continue;
    //                     }
    //                 } else {
    //                     continue;
    //                 }
    //             }
    //             match unit_type(filename.as_str())? {
    //                 UnitType::Regular(_) => {
    //                     let parse = Self::load(file.path(), None)?;
    //                     results.push((filename, parse));
    //                 }
    //                 UnitType::Template(template_name) => {
    //                     let template_name = template_name.to_owned();
    //                     let mut handle = File::open(file.path()).context(ReadFileSnafu {
    //                         path: path_str.to_owned(),
    //                     })?;
    //                     let mut content = String::new();
    //                     handle.read_to_string(&mut content).context(ReadFileSnafu {
    //                         path: path_str.to_owned(),
    //                     })?;

    //                     templates.insert(template_name, content);
    //                 }
    //                 UnitType::Instance(instance_name, template_name) => {
    //                     let template_name = template_name.to_owned();
    //                     match instances.get_mut(template_name.as_str()) {
    //                         Some(current) => {
    //                             current.push(instance_name.to_string());
    //                         }
    //                         None => {
    //                             instances.insert(template_name, vec![instance_name.to_string()]);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     Ok(())
    // }

    fn __parse_unit(__source: UnitParser, __from: Option<&Self>) -> Result<Self>;

    fn __load<S: AsRef<Path>>(
        path: S,
        filename: &str,
        from: Option<&Self>,
        root: bool,
    ) -> Result<Self> {
        let context = SpecifierContext::new(root);
        let path = path.as_ref();
        let mut file = File::open(path).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let mut content = String::new();
        file.read_to_string(&mut content).context(ReadFileSnafu {
            path: path.to_string_lossy().to_string(),
        })?;
        let parser =
            crate::parser::UnitParser::new(content.as_ref(), Rc::new(context), filename, path)?;
        Self::__parse_unit(parser, from)
    }

    fn load<S: AsRef<Path>>(path: S, root: bool) -> Result<Self> {
        let path = path.as_ref();
        Self::__load(
            path,
            path.file_name()
                .map_or("".to_string(), |x| x.to_string_lossy().to_string())
                .as_str(),
            None,
            root,
        )
    }

    fn load_named<S: AsRef<str>, P: AsRef<Path>>(
        paths: Vec<P>,
        name: S,
        root: bool,
    ) -> Result<Self> {
        // return when first one is found?
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
            UnitType::Regular(_) => fullname.as_str(),
        };
        let mut result = None;

        // load itself
        for dir in paths.iter() {
            let dir = dir.as_ref();
            let mut path = dir.to_owned();
            path.push(actual_file_name);
            if let Ok(res) = Self::__load(path, fullname.as_str(), None, root) {
                result = Some(res);
                break;
            }
        }

        // load drop-ins
        let mut dropin_dir_names = vec![
            format!("{}.d", Self::SUFFIX),
            format!("{}.d", fullname.as_str()),
        ];
        let segments: Vec<&str> = fullname.split('-').collect();
        for i in (1..segments.len()).rev() {
            let segmented = segments[0..i].join("-");
            let dir_name = format!("{}-.{}", segmented, Self::SUFFIX);
            dropin_dir_names.push(dir_name);
        }

        for dir_name in dropin_dir_names.iter() {
            for dir in paths.iter() {
                let dir = dir.as_ref();
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
                                        if let Ok(res) = Self::__load(
                                            entry.path(),
                                            fullname.as_str(),
                                            result.as_ref(),
                                            root,
                                        ) {
                                            result = Some(res);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        result.ok_or(Error::NoUnitFoundError {
            name: name.to_string(),
        })
    }
}

pub trait UnitSection: Sized + Clone {
    fn __parse_section(__source: SectionParser, __from: Option<Self>) -> Result<Option<Self>>;
}

pub trait UnitEntry: Sized + Clone {
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
