use crate::error::Error;
use crate::escape::escape;
use crate::template::{unit_type, UnitType};
use nix::sys::utsname::UtsName;
use nix::unistd::{Uid, User};
use nix::{
    sys::utsname::uname,
    unistd::{Gid, Group},
};
use os_release::OsRelease;
use std::{env, fs, path::Path};

pub(crate) struct SpecifierContext {
    os_release: OsRelease,
    uts: UtsName,
    root: bool,
}

impl SpecifierContext {
    pub(crate) fn new(root: bool) -> Self {
        let os_release = OsRelease::new().expect("Failed to read os-release.");
        let uts = uname().expect("Failed to read system information.");
        Self {
            os_release,
            root,
            uts,
        }
    }
}

// return Cow?
pub(crate) fn resolve(
    specifier: char,
    context: &SpecifierContext,
    filename: &str,
    path: &Path,
) -> Result<String, Error> {
    let result = match specifier {
        'a' => context.uts.machine().to_string_lossy().to_string(),
        'A' => context
            .os_release
            .extra
            .get("IMAGE_VERSION")
            .unwrap_or(&"".to_string())
            .into(),
        'b' => {
            fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("Failed to read boot_id.")
        }
        'B' => context
            .os_release
            .extra
            .get("BUILD_ID")
            .unwrap_or(&"".to_string())
            .into(),
        'C' => {
            if context.root {
                "/var/cache".into()
            } else {
                env::var("XDG_CACHE_HOME").unwrap_or("~/.cache".to_string())
            }
        }
        'd' => env::var("CREDENTIALS_DIRECTORY").unwrap_or("".to_string()),
        'E' => {
            if context.root {
                "/etc".into()
            } else {
                env::var("XDG_CONFIG_HOME").unwrap_or("~/.config".to_string())
            }
        }
        'f' => filename.to_string(),
        'g' => {
            if context.root {
                "root".into()
            } else {
                Group::from_gid(Gid::current())
                    .expect("Failed to read current group name.")
                    .map_or("".into(), |x| x.name)
            }
        }
        'G' => {
            if context.root {
                "0".into()
            } else {
                Gid::current().to_string()
            }
        }
        'h' => {
            if context.root {
                "/root".into()
            } else {
                env::var("HOME").unwrap_or("~".to_string())
            }
        }
        'H' => context.uts.nodename().to_string_lossy().to_string(),
        'i' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                escape(instance_name)
            } else {
                "".to_string()
            }
        }
        'I' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                instance_name.to_string()
            } else {
                "".to_string()
            }
        }
        'j' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                escape(instance_name.split('-').last().unwrap())
            } else {
                escape(
                    filename
                        .split('.')
                        .nth(0)
                        .unwrap()
                        .split('-')
                        .last()
                        .unwrap(),
                )
            }
        }
        'J' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                instance_name.split('-').last().unwrap().to_string()
            } else {
                filename
                    .split('.')
                    .nth(0)
                    .unwrap()
                    .split('-')
                    .last()
                    .unwrap()
                    .to_string()
            }
        }
        'l' => context
            .uts
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        'L' => {
            if context.root {
                "/var/log".into()
            } else {
                env::var("XDG_STATE_HOME")
                    .map_or("~/.local/state/log".into(), |x| format!("{x}/log"))
            }
        }
        'm' => fs::read_to_string("/etc/machine-id").expect("Failed to read machine-id."),
        'M' => context
            .os_release
            .extra
            .get("IMAGE_ID")
            .unwrap_or(&"".to_string())
            .into(),
        'n' => escape(filename),
        'N' => escape(filename.split(".").nth(0).unwrap()),
        'o' => context.os_release.id.to_string(),
        'p' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                escape(instance_name)
            } else {
                escape(filename.split('.').nth(0).unwrap())
            }
        }
        'P' => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                instance_name.to_string()
            } else {
                filename.split('.').nth(0).unwrap().to_string()
            }
        }
        'q' => context
            .uts
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        's' => env::var("SHELL").unwrap_or("".to_string()),
        'S' => {
            if context.root {
                "/var/lib".into()
            } else {
                env::var("XDG_STATE_HOME").unwrap_or("~/.local/share".to_string())
            }
        }
        't' => {
            if context.root {
                "/run".into()
            } else {
                env::var("XDG_RUNTIME_DIR").unwrap_or(format!("/run/user/{}", Uid::current()))
            }
        }
        'T' => env::var("TMPDIR")
            .unwrap_or(env::var("TEMP").unwrap_or(env::var("TMP").unwrap_or("/tmp".to_string()))),
        'u' => User::from_uid(Uid::current())
            .expect("Failed to read user name.")
            .map_or("".to_string(), |x| x.name),
        'U' => Uid::current().to_string(),
        'v' => context.uts.release().to_string_lossy().to_string(),
        'V' => env::var("TMPDIR").unwrap_or(
            env::var("TEMP").unwrap_or(env::var("TMP").unwrap_or("/var/tmp".to_string())),
        ),
        'w' => context.os_release.version_id.to_string(),
        'W' => context
            .os_release
            .extra
            .get("VARIANT_ID")
            .unwrap_or(&"".to_string())
            .into(),
        'y' => path.to_string_lossy().to_string(),
        'Y' => path
            .parent()
            .expect("Invalid file path.")
            .to_string_lossy()
            .to_string(),
        '%' => "%".to_string(),
        _ => "".into(),
    };
    Ok(result)
}
