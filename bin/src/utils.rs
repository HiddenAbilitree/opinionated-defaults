use {
  log::{error, info, trace},
  std::{env::current_dir, path::PathBuf},
};

fn ancestors() -> Option<impl Iterator<Item = PathBuf>> {
  current_dir()
    .map_err(|_| error!("Could not read current dir, probably no permissions."))
    .ok()
    .map(|start| std::iter::successors(Some(start), |p| p.parent().map(PathBuf::from)))
}

pub fn find_file(filename: &str) -> Option<PathBuf> {
  ancestors()?.find_map(|dir| {
    let path = dir.join(filename);
    trace!("Checking: {}", path.display());
    path.exists().then(|| {
      info!("Found {}", path.display());
      path
    })
  })
}

pub fn find_first_file(filenames: &[&str]) -> Option<PathBuf> {
  ancestors()?.find_map(|dir| {
    filenames.iter().find_map(|filename| {
      let path = dir.join(filename);
      trace!("Checking: {}", path.display());
      path.exists().then(|| {
        info!("Found {}", path.display());
        path
      })
    })
  })
}
