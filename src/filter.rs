use std::{alloc::GlobalAlloc, env::consts::OS, str::FromStr};

use miette::{IntoDiagnostic, Result};
use semver::Version;
use sysinfo::{RefreshKind, System};

use crate::{
    GlobalArgs,
    config::{Condition, OsName, OsType},
};

pub struct SystemInfo {
    pub os: OsName,
    pub distro: String,
    pub version: Option<semver::Version>,
    pub distro_like: Vec<String>,
}

pub fn get_system_info() -> Result<SystemInfo> {
    let os = OsName::from_str(OS).into_diagnostic()?;
    let distro = sysinfo::System::distribution_id();
    let distro_like = sysinfo::System::distribution_id_like();
    let version = sysinfo::System::os_version()
        .map(|v| Version::parse(&v))
        .and_then(Result::ok);

    Ok(SystemInfo {
        os,
        distro,
        version,
        distro_like,
    })
}

pub fn check_condition(
    system: &SystemInfo,
    condition: &Condition,
    global_args: &GlobalArgs,
) -> bool {
    // Check label
    if let Some(label) = &condition.label {
        if global_args.labels.is_empty() && condition.default == Some(true) {
            return true;
        }
        return global_args.labels.contains(label);
    }

    // Check OS conditions
    if let Some(condition_os_list) = &condition.os {
        let os_matches = condition_os_list.iter().any(|os| {
            // First check if the OS name matches
            if os.name() != system.os {
                return false;
            }
            // Then check OS-specific constraints
            match os {
                OsType::MacOS { version } => {
                    if let Some(version_req) = version {
                        match system.version {
                            Some(ref system_version) => version_req.matches(system_version),
                            None => false,
                        }
                    } else {
                        // No version constraint, so any macOS matches
                        true
                    }
                }
                OsType::Linux {
                    distro,
                    distro_like,
                } => {
                    // If neither distro nor distro_like is specified, any Linux matches
                    if distro.is_none() && distro_like.is_none() {
                        return true;
                    }

                    // Check if system distro matches any of the specified distros
                    let distro_matches = if let Some(distros) = distro {
                        distros.contains(&system.distro)
                    } else {
                        false
                    };

                    // Check if any of the system's distro_like values match any of the condition's distro_like values
                    let distro_like_matches = if let Some(condition_distro_likes) = distro_like {
                        // Check if there's any overlap between the two lists
                        condition_distro_likes.iter().any(|cond_like| {
                            system
                                .distro_like
                                .iter()
                                .any(|sys_like| cond_like == sys_like)
                        })
                    } else {
                        false
                    };

                    distro_matches || distro_like_matches
                }
                OsType::Windows => true,
            }
        });
        if !os_matches {
            return false;
        }
    }
    // Add other condition checks (architecture, hostname_pattern) here if needed
    // For now, if OS matches (or no OS constraint), return true
    true
}
