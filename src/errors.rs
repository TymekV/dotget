use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("unable to read config")]
#[diagnostic(code(config::read_fail), help("Ensure that the file {path:?} exists."))]
pub struct UnableToReadConfig {
    pub path: PathBuf,
}

#[derive(Error, Debug, Diagnostic)]
#[error("invalid config: {0}")]
#[diagnostic(
    code(config::deserialization_failed),
    help("Ensure that the syntax is correct and all of the required fields are provided.")
)]
pub struct InvalidConfig(pub serde_yaml::Error);

#[derive(Error, Debug, Diagnostic)]
#[error("{manager}: platform not supported")]
#[diagnostic(
    code(package_manager::platform_not_supported),
    help(
        r#"This package manager does not support your platform.
        Update conditions in your configuration file to avoid missmatches like this.
        Use `lint` command to catch most of these errors early."#
    )
)]
pub struct UnsupportedPlatform {
    pub manager: &'static str,
}
