use {
  ignore::Walk,
  log::{error, info, trace},
  regex::RegexSet,
  std::{env::current_dir, fs::read_to_string, path::PathBuf},
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

pub fn find_files(filename: &str) -> Vec<PathBuf> {
  let Some(ancestors) = ancestors() else {
    return Vec::new();
  };

  ancestors
    .filter_map(|dir| {
      let path = dir.join(filename);
      trace!("Checking: {}", path.display());
      path.exists().then(|| {
        info!("Found {}", path.display());
        path
      })
    })
    .collect()
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

pub fn find_tailwind_file() -> Option<PathBuf> {
  let base_path = current_dir().ok()?;
  let tailwind_regex =
    RegexSet::new([r#"@import ["']tailwindcss["'];"#, r#"@tailwind base;"#]).unwrap();

  Walk::new(&base_path)
    .filter_map(Result::ok)
    .filter(|entry| entry.file_type().is_some_and(|ft| ft.is_file()))
    .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "css"))
    .find_map(|entry| {
      let path = entry.path();
      let contents = read_to_string(path).ok()?;

      if tailwind_regex.is_match(&contents) {
        let stripped = path.strip_prefix(&base_path).ok()?;
        info!("Found tailwind file: {}", stripped.display());
        Some(stripped.to_path_buf())
      } else {
        None
      }
    })
}
