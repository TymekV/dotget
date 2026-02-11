use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Config {
    pub conditions: HashMap<String, Condition>,
}

/// Operating system type constraint.
#[derive(Deserialize, JsonSchema, Debug, Clone)]
#[serde(tag = "os", rename_all = "lowercase")]
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

/// Execution condition used to determine whether something
/// applies to the current system.
///
/// All fields are optional.
/// If a field is left empty, it does not restrict matching.
#[derive(Deserialize, JsonSchema, Debug, Clone)]
pub struct Condition {
    #[serde(flatten)]
    pub os: Option<OsType>,

    /// Processor architecture constraint.
    pub architecture: Option<String>,

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
    pub packages: HashMap<String, Vec<String>>,
}
