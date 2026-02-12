use clap::Parser;
use miette::Result;

use crate::{GlobalArgs, config::read_config, package_managers::PackageManagers};

#[derive(Parser, Debug, Clone)]
pub struct ApplyArgs {
    /// Perform a dry run without actually modifying anything on your system
    #[arg(long)]
    pub dry_run: bool,
}

pub async fn apply(
    managers: PackageManagers,
    global_args: GlobalArgs,
    args: ApplyArgs,
) -> Result<()> {
    let config = read_config(&global_args.file).await?;
    managers
        .get_installed(crate::package_managers::PackageManagerName::Pacman)
        .await?;
    Ok(())
}
