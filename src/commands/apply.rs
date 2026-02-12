use clap::Parser;
use miette::Result;

use crate::{
    GlobalArgs,
    config::read_config,
    filter::{check_condition, get_system_info},
    package_managers::PackageManagers,
};

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

    let system = get_system_info()?;

    let matching_groups = config.groups.into_iter().filter(|group| {
        group
            .conditions
            .iter()
            .filter_map(|condition_name| config.conditions.get(condition_name))
            .all(|c| check_condition(&system, c))
    });

    for group in matching_groups {
        for (manager, packages) in group.packages {
            managers.install_missing(manager, packages).await?;
        }
    }

    Ok(())
}
