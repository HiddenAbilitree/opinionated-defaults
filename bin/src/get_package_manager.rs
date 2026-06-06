use {
  crate::{
    types::{PackageJSON, PackageManager, Packages, ProjectData},
    utils::{find_file, find_first_file},
  },
  log::info,
  std::fs::read_to_string,
};

fn read_package_json_packages() -> Option<Packages> {
  let path = find_file("package.json")?;
  let content = read_to_string(&path).ok()?;
  let data: PackageJSON = serde_json::from_str(&content).ok()?;

  Some(
    data
      .dependencies
      .into_iter()
      .chain(data.dev_dependencies)
      .chain(data.peer_dependencies)
      .collect(),
  )
}

pub fn get_package_manager_data() -> Option<ProjectData> {
  let lockfiles: Vec<_> = PackageManager::ALL.iter().map(|pm| pm.lockfile()).collect();
  let path = find_first_file(&lockfiles)?;

  let manager = PackageManager::from_lockfile(path.file_name()?);
  info!("Lockfile path: {}", path.display());

  let content = read_to_string(&path).ok()?;
  let packages = manager.parse_lockfile(&content).map_or_else(
    |_| {
      eprintln!("❌ Could not parse the lockfile. Falling back to defaults...");
      Packages::new()
    },
    |data| data.packages,
  );

  let packages = read_package_json_packages().unwrap_or(packages);

  Some(ProjectData { packages, manager })
}
