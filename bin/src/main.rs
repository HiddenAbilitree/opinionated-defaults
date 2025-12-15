// TODO: switch to wrapped-wrapped config object for eslint ({ prettier: true } or smth)

mod generate_config;
mod get_package_manager;
mod handle_dependencies;
mod types;
mod utils;

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

fn default_project() -> ProjectData {
  warn!("Could not find an existing package manager, defaulting to bun...");
  ProjectData {
    packages: Packages::new(),
    manager: PackageManager::Bun,
  }
}

fn run_install(manager: PackageManager) -> bool {
  if manager.command().output().is_err() {
    eprintln!(
      "❌ Could not install @hiddenability/opinionated-defaults@latest with {}.",
      manager.cli()
    );
    return false;
  }
  true
}

fn main() -> Result<()> {
  let start = Instant::now();
  env_logger::init();

  let mut project = get_package_manager_data().unwrap_or_else(default_project);

  if !run_install(project.manager) {
    return Ok(());
  }

  if project.manager == PackageManager::BunOld {
    if !run_install(project.manager) {
      return Ok(());
    }
    project = get_package_manager_data().unwrap_or_else(default_project);
  }

  generate_config(project.packages)?;

  println!(
    "✅ Done in {:.2?} using {}",
    start.elapsed(),
    project.manager.cli()
  );

  Ok(())
}
