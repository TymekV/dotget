pub mod pacman;

use std::collections::HashMap;

use async_trait::async_trait;
use miette::Result;
use owo_colors::OwoColorize;
use tracing::info;

use crate::{config::OsName, errors::UnsupportedPlatform};

#[derive(Clone)]
pub struct PackageMetadata {
    pub version: Option<String>,
}

#[async_trait]
pub trait PackageManager {
    const NAME: &'static str;
    const SUPPORTED_OS: &'static [OsName];

    async fn get_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        let error = UnsupportedPlatform {
            manager: Self::NAME,
        };
        Err(error.into())
    }

    async fn install(&self, _packages: Vec<String>) -> Result<()> {
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


                pub async fn get_installed(&self, manager: PackageManagerName) -> Result<HashMap<String, PackageMetadata>> {
                    match manager {
                        $(
                            PackageManagerName::$name => <$struct as PackageManager>::get_installed(&self.[< $name:lower >]).await
                        ),*
                    }
                }

                pub async fn install(&self, manager: PackageManagerName, packages: Vec<String>) -> Result<()> {
                    match manager {
                        $(
                            PackageManagerName::$name => <$struct as PackageManager>::install(&self.[< $name:lower >], packages).await
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

impl PackageManagers {
    pub async fn install_missing(
        &self,
        manager: PackageManagerName,
        packages: Vec<String>,
    ) -> Result<()> {
        let installed = self.get_installed(manager).await?;

        let missing = packages
            .into_iter()
            .filter(|package| !installed.contains_key(package))
            .collect::<Vec<_>>();

        info!("Found {} missing packages", missing.len().cyan().bold());

        self.install(manager, missing).await?;

        Ok(())
    }
}
