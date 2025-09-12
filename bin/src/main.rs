/*
 * TODO:
 * switch to wrapped-wrapped config object for eslint ({ prettier: true } or smth)
 */
use {
  crate::{
    generate_config::generate_config,
    get_package_manager::get_package_manager_data,
    types::{PackageManager, Packages, ProjectData},
  },
  anyhow::Result,
  log::warn,
  std::time::Instant,
};

mod generate_config;
mod get_package_manager;
mod handle_dependencies;
mod types;
mod utils;

fn main() -> Result<()> {
  let before = Instant::now();

  env_logger::init();

  let project_data = get_package_manager_data().unwrap_or_else(|| {
    warn!("Could not find an existing package manager, defaulting to bun...");
    ProjectData {
      packages: Packages::new(),
      manager: PackageManager::Bun,
    }
  });

  if project_data.manager.command().output().is_err() {
    eprintln!(
      "❌ Could not install @hiddenability/opinionated-defaults@latest with {}.",
      project_data.manager.cli()
    );

    return Ok(());
  }

  generate_config(project_data.packages)?;

  println!(
    "✅ Done in {:.2?} using {}",
    before.elapsed(),
    project_data.manager.cli()
  );

  Ok(())
}
