use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::package_managers::PackageManagerName;

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Config {
    pub conditions: HashMap<String, Condition>,
    pub groups: Vec<Group>,
}

/// Operating system type constraint.
#[derive(Deserialize, JsonSchema, Debug, Clone)]
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
        #[serde(default)]
        distro: Vec<String>,

        /// Distribution family identifiers matched against the
        /// `ID_LIKE` field in `/etc/os-release`.
        ///
        /// This allows matching broader distribution families, e.g.:
        /// - `"debian"` (matches Ubuntu, Linux Mint, etc.)
        /// - `"rhel"` (matches Fedora, Rocky, AlmaLinux, etc.)
        ///
        /// If multiple values are provided, they are treated as a logical OR.
        #[serde(default)]
        distro_like: Vec<String>,
    },
}

/// Execution condition used to determine whether something
/// applies to the current system.
///
/// All fields are optional.
/// If a field is left empty, it does not restrict matching.
#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Condition {
    /// Operating system constraints.
    /// Works like a logical OR.
    pub os: Vec<OsType>,

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
    pub conditions: Vec<String>,
    pub packages: HashMap<PackageManagerName, Vec<String>>,
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
