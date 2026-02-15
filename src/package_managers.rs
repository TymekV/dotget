pub mod pacman;

use std::collections::HashMap;

use async_trait::async_trait;
use miette::Result;
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::info;

use crate::{config::OsName, errors::UnsupportedPlatform};

#[async_trait]
pub trait PackageManager {
    const NAME: &'static str;
    const SUPPORTED_OS: &'static [OsName];

    type Options: for<'de> Deserialize<'de> + JsonSchema + Send + Sync + Clone;
    type Package: for<'de> Deserialize<'de> + JsonSchema + Send + Sync + Clone;

    async fn get_installed(&self) -> Result<HashMap<String, Self::Package>> {
        let error = UnsupportedPlatform {
            manager: Self::NAME,
        };
        Err(error.into())
    }

    fn filter_missing(
        &self,
        _installed: HashMap<String, Self::Package>,
        _desired: &Self::Options,
    ) -> Result<(Self::Options, usize)> {
        let error = UnsupportedPlatform {
            manager: Self::NAME,
        };
        Err(error.into())
    }

    async fn install_missing(&self, config: Self::Options) -> Result<()> {
        let installed = self.get_installed().await?;

        let (missing, count) = self.filter_missing(installed, &config)?;
        if count == 0 {
            info!("No missing packages");
            return Ok(());
        }
        info!("Found {} missing packages", count.blue().bold());

        self.install(missing).await?;

        Ok(())
    }

    async fn install(&self, _options: Self::Options) -> Result<()> {
        let error = UnsupportedPlatform {
            manager: Self::NAME,
        };
        Err(error.into())
    }
}

macro_rules! package_managers {
    ($( $name:ident => $struct:ty ),* $(,)?) => {
        #[derive(serde::Deserialize, schemars::JsonSchema, Debug, Clone, Eq, PartialEq, Hash, Copy)]
        #[serde(rename_all = "lowercase")]
        pub enum PackageManagerName {
            $($name),*
        }

        #[derive(serde::Deserialize, schemars::JsonSchema, strum::Display, Debug, Clone)]
        #[serde(tag = "manager", content = "install", rename_all = "lowercase")]
        pub enum PackageManagerConfig {
            $($name(<$struct as PackageManager>::Options)),*
        }

        paste::paste! {
            #[derive(Clone)]
            pub struct PackageManagers {
                $(
                    [< $name:lower >]: std::sync::Arc<$struct>
                ),*
            }

            impl PackageManagers {
                pub fn new() -> miette::Result<Self> {
                    let managers = Self {
                        $(
                            [< $name: lower >]: std::sync::Arc::new($struct::new()?)
                        ),*
                    };
                    Ok(managers)
                }


                pub async fn install_missing(&self, config: PackageManagerConfig) -> Result<()> {
                    match config {
                        $(
                            PackageManagerConfig::$name(options) => <$struct as PackageManager>::install_missing(&self.[< $name:lower >], options).await
                        ),*
                    }
                }
            }
        }
    };
}

package_managers!(
    Pacman => pacman::Pacman,
);
