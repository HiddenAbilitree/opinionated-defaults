mod generate_config;
mod get_package_manager;
mod handle_dependencies;
mod types;
mod utils;

use {
  crate::{
    generate_config::generate_config,
    get_package_manager::get_package_manager_data,
    types::{PackageManager, Packages, ProjectData, Tooling},
  },
  anyhow::Result,
  dialoguer::{Select, theme::ColorfulTheme},
  log::warn,
  std::{env, time::Instant},
};

fn default_project() -> ProjectData {
  warn!("Could not find an existing package manager, defaulting to bun...");
  ProjectData {
    packages: Packages::new(),
    manager: PackageManager::Bun,
  }
}

fn run_install(manager: PackageManager, tooling: Tooling) -> bool {
  if manager.command(tooling).output().is_err() {
    eprintln!("❌ Could not install dependencies with {}.", manager.cli());
    return false;
  }
  true
}

fn get_tooling() -> Result<Tooling> {
  let args: Vec<String> = env::args().collect();

  if args.iter().any(|a| a == "-ox") {
    return Ok(Tooling::Ox);
  }

  if args.iter().any(|a| a == "-es") {
    return Ok(Tooling::Eslint);
  }

  let options = &["ESLint + Prettier", "Oxlint + Oxfmt"];
  let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Which tooling would you like to use?")
    .items(options)
    .default(0)
    .interact()?;

  Ok(match selection {
    1 => Tooling::Ox,
    _ => Tooling::Eslint,
  })
}

fn main() -> Result<()> {
  let start = Instant::now();
  env_logger::init();

  let tooling = get_tooling()?;

  let mut project = get_package_manager_data().unwrap_or_else(default_project);

  if !run_install(project.manager, tooling) {
    return Ok(());
  }

  if project.manager == PackageManager::BunOld {
    if !run_install(project.manager, tooling) {
      return Ok(());
    }
    project = get_package_manager_data().unwrap_or_else(default_project);
  }

  generate_config(project.packages, tooling)?;

  println!(
    "✅ Done in {:.2?} using {}",
    start.elapsed(),
    project.manager.cli()
  );

  Ok(())
}
