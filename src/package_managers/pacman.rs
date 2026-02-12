use std::collections::HashMap;

use async_trait::async_trait;
use miette::{IntoDiagnostic, Result};

#[cfg(target_os = "linux")]
use alpm::Alpm;

use crate::{
    config::OsName,
    package_managers::{PackageManager, PackageMetadata},
};

pub struct Pacman {}

impl Pacman {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[async_trait]
impl PackageManager for Pacman {
    const NAME: &'static str = "pacman";
    const SUPPORTED_OS: &'static [OsName] = &[OsName::Linux];

    #[cfg(target_os = "linux")]
    async fn get_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let alpm = Alpm::new("/", "/var/lib/pacman").into_diagnostic()?;
        let db = alpm.localdb();

        let packages = db
            .pkgs()
            .iter()
            .map(|package| {
                (
                    package.name().to_string(),
                    PackageMetadata {
                        version: Some(package.version().to_string()),
                    },
                )
            })
            .collect();

        Ok(packages)
    }

    #[cfg(target_os = "linux")]
    async fn install(&self, packages: Vec<String>) -> Result<()> {
        todo!()
    }
}
