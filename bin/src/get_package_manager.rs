use {
  crate::{
    types::{PackageManager, Packages, ProjectData},
    utils::find_first_file,
  },
  log::info,
  std::fs::read_to_string,
};

pub fn get_package_manager_data() -> Option<ProjectData> {
  let lockfiles: Vec<_> = PackageManager::ALL.iter().map(|pm| pm.lockfile()).collect();
  let path = find_first_file(&lockfiles)?;

  let manager = PackageManager::from_lockfile(path.file_name()?);
  info!("Lockfile path: {}", path.display());

  let content = read_to_string(&path).ok()?;
  let packages = manager
    .parse_lockfile(&content)
    .map(|data| data.packages)
    .unwrap_or_else(|_| {
      eprintln!("❌ Could not parse the lockfile. Falling back to defaults...");
      Packages::new()
    });

  Some(ProjectData { packages, manager })
}
