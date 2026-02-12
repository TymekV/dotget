use miette::Result;

use crate::{GlobalArgs, config::read_config, package_managers::PackageManagers};

pub async fn lint(managers: PackageManagers, global_args: GlobalArgs) -> Result<()> {
    let config = read_config(&global_args.file).await?;
    todo!()
}
