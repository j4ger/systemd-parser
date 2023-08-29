use crate::error::Error;
use crate::escape::escape;
use crate::{
    parser::Rule,
    template::{unit_type, UnitType},
};
use nix::unistd::{Uid, User};
use nix::{
    sys::utsname::uname,
    unistd::{Gid, Group},
};
use os_release::OsRelease;
use std::{env, fs, path::Path};

pub(crate) struct SpecifierContext {
    os_release: OsRelease,
    root: bool,
}

impl SpecifierContext {
    pub(crate) fn new(root: bool) -> Self {
        let os_release = OsRelease::new().expect("Failed to read os-release.");
        Self { os_release, root }
    }
}

// return Cow?
pub(crate) fn resolve(
    rule: &Rule,
    context: &SpecifierContext,
    filename: &str,
    path: &Path,
) -> Result<String, Error> {
    let uts = uname().expect("Failed to read system information.");
    let result = match rule {
        Rule::architecture => uts.machine().to_string_lossy().to_string(),
        Rule::os_image_version => context
            .os_release
            .extra
            .get("IMAGE_VERSION")
            .unwrap_or(&"".to_string())
            .into(),
        Rule::boot_id => {
            fs::read_to_string("/proc/sys/kernel/random/boot_id").expect("Failed to read boot_id.")
        }
        Rule::os_build_id => context
            .os_release
            .extra
            .get("BUILD_ID")
            .unwrap_or(&"".to_string())
            .into(),
        Rule::cache_root => {
            if context.root {
                "/var/cache".into()
            } else {
                env::var("XDG_CACHE_HOME").unwrap_or("~/.cache".to_string())
            }
        }
        Rule::credentials_dir => env::var("CREDENTIALS_DIRECTORY").unwrap_or("".to_string()),
        Rule::unescaped_filename => filename.to_string(),
        Rule::user_gid => {
            if context.root {
                "0".into()
            } else {
                Gid::current().to_string()
            }
        }
        Rule::user_group => {
            if context.root {
                "root".into()
            } else {
                Group::from_gid(Gid::current())
                    .expect("Failed to read current group name.")
                    .map_or("".into(), |x| x.name)
            }
        }
        Rule::user_home => {
            if context.root {
                "/root".into()
            } else {
                env::var("HOME").unwrap_or("~".to_string())
            }
        }
        Rule::host_name => uts.nodename().to_string_lossy().to_string(),
        Rule::instance_name => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                escape(instance_name)
            } else {
                "".to_string()
            }
        }
        Rule::unescaped_instance_name => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                instance_name.to_string()
            } else {
                "".to_string()
            }
        }
        Rule::final_prefix_component => {
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
        Rule::unescaped_final_prefix_component => {
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
        Rule::short_host_name => uts
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        Rule::log_root => {
            if context.root {
                "/var/log".into()
            } else {
                env::var("XDG_STATE_HOME")
                    .map_or("~/.local/state/log".into(), |x| format!("{x}/log"))
            }
        }
        Rule::machine_id => {
            fs::read_to_string("/etc/machine-id").expect("Failed to read machine-id.")
        }
        Rule::os_image_id => context
            .os_release
            .extra
            .get("IMAGE_ID")
            .unwrap_or(&"".to_string())
            .into(),
        Rule::full_unit_name => escape(filename),
        Rule::full_unit_name_without_suffix => escape(filename.split(".").nth(0).unwrap()),
        Rule::os_id => context.os_release.id.to_string(),
        Rule::prefix => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                escape(instance_name)
            } else {
                escape(filename.split('.').nth(0).unwrap())
            }
        }
        Rule::unescaped_prefix => {
            if let UnitType::Instance(instance_name, _) = unit_type(filename)? {
                instance_name.to_string()
            } else {
                filename.split('.').nth(0).unwrap().to_string()
            }
        }
        Rule::pretty_host_name => uts
            .nodename()
            .to_string_lossy()
            .split('.')
            .nth(0)
            .unwrap()
            .to_string(),
        Rule::shell => env::var("SHELL").unwrap_or("".to_string()),
        Rule::state_root => {
            if context.root {
                "/var/lib".into()
            } else {
                env::var("XDG_STATE_HOME").unwrap_or("~/.local/share".to_string())
            }
        }
        Rule::runtime_root => {
            if context.root {
                "/run".into()
            } else {
                env::var("XDG_RUNTIME_DIR").unwrap_or(format!("/run/user/{}", Uid::current()))
            }
        }
        Rule::temp_root => env::var("TMPDIR")
            .unwrap_or(env::var("TEMP").unwrap_or(env::var("TMP").unwrap_or("/tmp".to_string()))),
        Rule::user_name => User::from_uid(Uid::current())
            .expect("Failed to read user name.")
            .map_or("".to_string(), |x| x.name),
        Rule::user_uid => Uid::current().to_string(),
        Rule::kernel_release => uts.release().to_string_lossy().to_string(),
        Rule::persist_temp_dir => env::var("TMPDIR").unwrap_or(
            env::var("TEMP").unwrap_or(env::var("TMP").unwrap_or("/var/tmp".to_string())),
        ),
        Rule::os_version_id => context.os_release.version_id.to_string(),
        Rule::os_variant_id => context
            .os_release
            .extra
            .get("VARIANT_ID")
            .unwrap_or(&"".to_string())
            .into(),
        Rule::fragment_path => path.to_string_lossy().to_string(),
        Rule::fragment_dir => path
            .parent()
            .expect("Invalid file path.")
            .to_string_lossy()
            .to_string(),
        Rule::escape_percentage => "%".to_string(),
        _ => "".into(),
    };
    Ok(result)
}
