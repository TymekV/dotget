use std::{collections::HashMap, path::Path};

use miette::Result;
use schemars::JsonSchema;
use serde::Deserialize;
use strum::EnumString;
use tokio::fs;

use crate::{
    errors::{InvalidConfig, UnableToReadConfig},
    package_managers::PackageManagerConfig,
};

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Config {
    pub conditions: HashMap<String, Condition>,
    pub groups: Vec<Group>,
}

#[derive(Deserialize, JsonSchema, EnumString, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum OsName {
    Windows,
    MacOS,
    Linux,
}

/// Operating system type constraint.
#[derive(Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OsType {
    Windows,
    MacOS {
        /// Optional semantic version requirement for the macOS version.
        ///
        /// This is evaluated against the system's macOS version
        /// (e.g. `13.5.1`).
        ///
        /// Examples:
        /// - `">=13.0.0"` — macOS Ventura or newer
        /// - `"^14.0.0"` — any macOS 14 release
        /// - `"<12.0.0"` — older than macOS Monterey
        #[schemars(with = "Option<String>")]
        version: Option<semver::VersionReq>,
    },
    Linux {
        /// Distribution identifiers matched against the `ID` field in `/etc/os-release`.
        ///
        /// Examples:
        /// - `"arch"`
        /// - `"ubuntu"`
        /// - `"fedora"`
        ///
        /// If multiple values are provided, they are treated as a logical OR.
        distro: Option<Vec<String>>,

        /// Distribution family identifiers matched against the
        /// `ID_LIKE` field in `/etc/os-release`.
        ///
        /// This allows matching broader distribution families, e.g.:
        /// - `"debian"` (matches Ubuntu, Linux Mint, etc.)
        /// - `"rhel"` (matches Fedora, Rocky, AlmaLinux, etc.)
        ///
        /// If multiple values are provided, they are treated as a logical OR.
        distro_like: Option<Vec<String>>,
    },
}

impl OsType {
    pub fn name(&self) -> OsName {
        match self {
            OsType::Windows => OsName::Windows,
            OsType::MacOS { .. } => OsName::MacOS,
            OsType::Linux { .. } => OsName::Linux,
        }
    }
}

/// Execution condition used to determine whether something
/// applies to the current system.
///
/// All fields are optional.
/// If a field is left empty, it does not restrict matching.
#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Condition {
    /// Custom label passed to `apply`.
    /// If set, must be passed to activate this condition.
    pub label: Option<String>,

    /// If set to `true`, will activate when no labels are provided.
    pub default: Option<bool>,

    /// Operating system constraints.
    /// Works like a logical OR.
    pub os: Option<Vec<OsType>>,

    /// Processor architecture constraints.
    /// Works like a logical OR.
    pub architecture: Option<Vec<String>>,

    /// Hostname glob pattern constraint.
    ///
    /// Matching uses standard glob semantics:
    /// - `*` matches any sequence of characters (including empty)
    /// - `?` matches exactly one character
    /// - `[abc]` matches any character in the set
    /// - `[a-z]` matches any character in the range
    ///
    /// Examples:
    /// - `"laptop-*"` matches any hostname starting with "laptop-"
    /// - `"*.local"` matches any hostname ending with ".local"
    /// - `"build-??"` matches hostnames like "build-01", "build-AB"
    pub hostname_pattern: Option<String>,
}

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Group {
    pub name: Option<String>,
    pub conditions: Vec<String>,
    pub packages: Vec<PackageManagerConfig>,
}

pub async fn read_config(path: &Path) -> Result<Config> {
    let config = fs::read_to_string(path)
        .await
        .map_err(|_| UnableToReadConfig { path: path.into() })?;
    let config = serde_yaml::from_str::<Config>(&config).map_err(InvalidConfig)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use schemars::schema_for;

    use super::*;

    #[test]
    fn generate_schema() {
        let global_schema = schema_for!(Config);
        let global_schema =
            serde_json::to_string(&global_schema).expect("Failed to serialize schema");
        fs::write("schema.json", global_schema).expect("Failed to write schema");
    }
}
