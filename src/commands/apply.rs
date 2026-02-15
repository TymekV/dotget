use clap::Parser;
use miette::Result;
use owo_colors::OwoColorize;
use tracing::{info, instrument};

use crate::{
    GlobalArgs,
    config::read_config,
    filter::{check_condition, get_system_info},
    package_managers::{PackageManagerConfig, PackageManagers},
};

#[derive(Parser, Debug, Clone)]
pub struct ApplyArgs {
    /// Perform a dry run without actually modifying anything on your system
    #[arg(long)]
    pub dry_run: bool,
}

#[instrument(skip(managers))]
async fn install_batch(managers: &PackageManagers, batch: PackageManagerConfig) -> Result<()> {
    info!("Installing {} packages", batch.blue().bold());
    managers.install_missing(batch).await?;
    Ok(())
}

pub async fn apply(
    managers: PackageManagers,
    global_args: GlobalArgs,
    args: ApplyArgs,
) -> Result<()> {
    let config = read_config(&global_args.file).await?;

    let system = get_system_info()?;

    let matching_groups = config.groups.into_iter().filter(|group| {
        group
            .conditions
            .iter()
            .filter_map(|condition_name| config.conditions.get(condition_name))
            .all(|c| check_condition(&system, c))
    });

    for group in matching_groups {
        for batch in group.packages {
            install_batch(&managers, batch).await?;
        }
    }

    Ok(())
}
