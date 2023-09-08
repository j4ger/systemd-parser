use crate::error::Error;
use crate::escape::escape;
use crate::template::{unit_type, UnitType};
use nix::sys::utsname::UtsName;
use nix::unistd::{Uid, User};
use nix::{
    sys::utsname::uname,
    unistd::{Gid, Group},
};
use once_cell::sync::Lazy;
use os_release::OsRelease;
use std::{env, fs, path::Path};

static OS_RELEASE: Lazy<OsRelease> =
    Lazy::new(|| OsRelease::new().expect("Failed to read os-release."));
static UTS_NAME: Lazy<UtsName> = Lazy::new(|| uname().expect("Failed to read system information."));

// return Cow?
pub(crate) fn resolve(
    specifier: char,
    root: bool,
    filename: &str,
    path: &Path,
) -> Result<String, Error> {
    let result = match specifier {
        'a' => UTS_NAME.machine().to_string_lossy().to_string(),
        'A' => OS_RELEASE
            .extra
            .get("IMAGE_VERSION")
            .unwrap_or(&"".to_string())
            .into(),
        'b' => {
            fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("Failed to read boot_id.")
        }
        'B' => OS_RELEASE
            .extra
            .get("BUILD_ID")
            .unwrap_or(&"".to_string())
            .into(),
        'C' => {
            if root {
                "/var/cache".into()
            } else {
                env::var("XDG_CACHE_HOME").unwrap_or("~/.cache".to_string())
            }
        }
        'd' => env::var("CREDENTIALS_DIRECTORY").unwrap_or("".to_string()),
        'E' => {
            if root {
                "/etc".into()
            } else {
                env::var("XDG_CONFIG_HOME").unwrap_or("~/.config".to_string())
            }
        }
        'f' => filename.to_string(),
        'g' => {
            if root {
                "root".into()
            } else {
                Group::from_gid(Gid::current())
                    .expect("Failed to read current group name.")
                    .map_or("".into(), |x| x.name)
            }
        }
        'G' => {
            if root {
                "0".into()
            } else {
                Gid::current().to_string()
            }
        }
        'h' => {
            if root {
                "/root".into()
            } else {
                env::var("HOME").unwrap_or("~".to_string())
            }
        }
        'H' => UTS_NAME.nodename().to_string_lossy().to_string(),
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
        'l' => UTS_NAME
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        'L' => {
            if root {
                "/var/log".into()
            } else {
                env::var("XDG_STATE_HOME")
                    .map_or("~/.local/state/log".into(), |x| format!("{x}/log"))
            }
        }
        'm' => fs::read_to_string("/etc/machine-id").expect("Failed to read machine-id."),
        'M' => OS_RELEASE
            .extra
            .get("IMAGE_ID")
            .unwrap_or(&"".to_string())
            .into(),
        'n' => escape(filename),
        'N' => escape(filename.split(".").nth(0).unwrap()),
        'o' => OS_RELEASE.id.to_string(),
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
        'q' => UTS_NAME
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        's' => env::var("SHELL").unwrap_or("".to_string()),
        'S' => {
            if root {
                "/var/lib".into()
            } else {
                env::var("XDG_STATE_HOME").unwrap_or("~/.local/share".to_string())
            }
        }
        't' => {
            if root {
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
        'v' => UTS_NAME.release().to_string_lossy().to_string(),
        'V' => env::var("TMPDIR").unwrap_or(
            env::var("TEMP").unwrap_or(env::var("TMP").unwrap_or("/var/tmp".to_string())),
        ),
        'w' => OS_RELEASE.version_id.to_string(),
        'W' => OS_RELEASE
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
