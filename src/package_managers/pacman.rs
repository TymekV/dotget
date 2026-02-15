#[cfg(target_os = "linux")]
mod utils;

#[cfg(target_os = "linux")]
use alpm::Alpm;
use async_trait::async_trait;
use miette::{IntoDiagnostic, Result};
use schemars::JsonSchema;
use serde::Deserialize;

use std::collections::HashMap;

use crate::{config::OsName, package_managers::PackageManager};

pub struct Pacman {}

impl Pacman {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct PacmanPackage {
    pub name: String,
    pub version: String,
    pub aur: bool,
}

#[derive(Deserialize, Debug, Clone, JsonSchema)]
pub struct PacmanOptions {
    /// Packages installed using `pacman`
    pub repo: Option<Vec<String>>,

    /// Additional arguments passed to `pacman`
    pub pacman_args: Option<Vec<String>>,

    /// Packages installed using user's preferred AUR helper by default.
    pub aur: Option<Vec<String>>,

    /// Args passed to user's AUR helper.
    pub aur_helper_args: Option<Vec<String>>,

    /// Force the usage of a specified AUR helper.
    pub force_aur_helper: Option<String>,
}

#[async_trait]
impl PackageManager for Pacman {
    const NAME: &'static str = "pacman";
    const SUPPORTED_OS: &'static [OsName] = &[OsName::Linux];

    type Options = PacmanOptions;
    type Package = PacmanPackage;

    #[cfg(target_os = "linux")]
    async fn get_installed(&self) -> Result<HashMap<String, Self::Package>> {
        let alpm = Alpm::new("/", "/var/lib/pacman").into_diagnostic()?;
        let db = alpm.localdb();

        let packages = db
            .pkgs()
            .iter()
            .map(|package| {
                // dbg!(&package.origin());
                (
                    package.name().to_string(),
                    PacmanPackage {
                        name: package.name().to_string(),
                        version: package.version().to_string(),
                        aur: false, // TODO
                    },
                )
            })
            .collect();

        Ok(packages)
    }

    fn filter_missing(
        &self,
        installed: HashMap<String, Self::Package>,
        desired: &Self::Options,
    ) -> Result<(Self::Options, usize)> {
        let repo = desired.repo.clone().map(|r| {
            r.into_iter()
                .filter(|p| !installed.get(p).is_some_and(|package| !package.aur))
                .collect::<Vec<_>>()
        });

        let aur = desired.aur.clone().map(|r| {
            r.into_iter()
                .filter(|p| !installed.get(p).is_some_and(|package| package.aur))
                .collect::<Vec<_>>()
        });

        let missing_count =
            repo.as_ref().map_or(0, |r| r.len()) + aur.as_ref().map_or(0, |a| a.len());

        Ok((
            Self::Options {
                repo,
                aur,
                ..desired.clone()
            },
            missing_count,
        ))
    }

    #[cfg(target_os = "linux")]
    async fn install(&self, options: Self::Options) -> Result<()> {
        use crate::package_managers::pacman::utils::select_aur_helper;
        use miette::Context;
        use owo_colors::OwoColorize;
        use tokio::process::Command;
        use tracing::info;

        dbg!(&options);

        if let Some(repo_packages) = &options.repo
            && !repo_packages.is_empty()
        {
            use std::process::Stdio;
            let separator = ",".dimmed().to_string();
            info!(
                packages = repo_packages.join(&separator),
                "Installing {} repo packages",
                repo_packages.len().blue().bold()
            );

            let status = Command::new("sudo")
                .arg("pacman")
                .arg("-S")
                .arg("--needed") // Skip packages that are already up to date
                .args(options.pacman_args.unwrap_or_default())
                .args(repo_packages)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .await
                .into_diagnostic()
                .wrap_err("Failed to execute pacman")?;

            if !status.success() {
                miette::bail!("pacman exited with status: {}", status);
            }
        }

        if let Some(aur_packages) = &options.aur
            && !aur_packages.is_empty()
        {
            let helper = select_aur_helper(options.force_aur_helper.clone())
                .await
                .into_diagnostic()?;

            info!("Using {}", helper.blue().bold());

            let status = Command::new(&helper)
                .arg("-S")
                .arg("--needed")
                .args(options.aur_helper_args.unwrap_or_default())
                .args(aur_packages)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()
                .await
                .into_diagnostic()
                .wrap_err("Failed to execute {helper}")?;

            if !status.success() {
                miette::bail!("{helper} exited with status: {}", status);
            }
        }

        Ok(())
    }
}
