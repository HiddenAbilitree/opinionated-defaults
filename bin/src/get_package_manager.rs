use std::fs::read_to_string;

use log::{info, trace};

use crate::{
  types::{PackageManager, Packages, ProjectData},
  utils::find_first_file,
};

pub fn get_package_manager_data() -> Option<ProjectData> {
  let path = find_first_file(PackageManager::iterator().map(|pm| pm.lockfile()).collect())?;

  let package_manager = PackageManager::from_lockfile(path.file_name().unwrap());

  info!("Lockfile Path: {:#?}", path);

  let content = read_to_string(path).unwrap();

  // trace!("Lockfile Contents:\n{:#?}", content);

  Some(ProjectData {
    packages: package_manager
      .parse_lockfile(content)
      .map(|repo_data| repo_data.packages)
      .unwrap_or_else(|_| {
        eprintln!("❌ Could not parse the lockfile. Falling back to defaults..."); 
        Packages::new()
      }),
    manager: package_manager,
  })
}
