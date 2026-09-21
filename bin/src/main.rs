mod generate_config;
mod get_package_manager;
mod monorepo;
mod types;
mod utils;

use {
  crate::{
    generate_config::generate_config,
    get_package_manager::{get_package_manager_data, read_package_json_packages},
    monorepo::needs_solid_plugin,
    types::{PackageManager, ProjectData},
  },
  anyhow::Result,
  log::warn,
  std::{env, time::Instant},
};

fn default_project() -> ProjectData {
  warn!("Could not find an existing package manager, defaulting to bun...");
  ProjectData {
    packages: read_package_json_packages().unwrap_or_default(),
    manager: PackageManager::Bun,
  }
}

fn run_install(manager: PackageManager, solid: bool) -> bool {
  if manager.command(solid).output().is_err() {
    eprintln!("❌ Could not install dependencies with {}.", manager.cli());
    return false;
  }
  true
}

fn main() -> Result<()> {
  env_logger::init();

  let start = Instant::now();

  let mut project = get_package_manager_data().unwrap_or_else(default_project);
  let solid = needs_solid_plugin(&env::current_dir()?, &project.packages)?;

  if !run_install(project.manager, solid) {
    return Ok(());
  }

  if project.manager == PackageManager::BunOld {
    if !run_install(project.manager, solid) {
      return Ok(());
    }
    project = get_package_manager_data().unwrap_or_else(default_project);
  }

  generate_config(&project.packages, project.manager)?;

  println!(
    "✅ Done in {:.2?} using {}",
    start.elapsed(),
    project.manager.cli()
  );

  Ok(())
}
