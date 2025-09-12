use {
  ignore::Walk,
  log::{error, info, trace},
  regex::RegexSet,
  std::{env::current_dir, fs::read_to_string, path::PathBuf},
};

// from https://github.com/lbwa/package-json-rs/blob/main/src/fs.rs#L10-L25
pub fn find_file(filename: &str) -> Option<PathBuf> {
  let Ok(mut current_dir) = current_dir() else {
    error!("Could not read current dir, probably no permissions.");
    return None;
  };
  loop {
    let path = current_dir.join(filename);
    trace!("Current path: {:?}", path.to_str());
    if path.exists() {
      info!("Found {}", path.to_str().unwrap());
      return Some(path);
    }
    if !current_dir.pop() {
      return None;
    }
  }
}

pub fn find_first_file(filenames: Vec<&str>) -> Option<PathBuf> {
  let Ok(mut current_dir) = current_dir() else {
    error!("Could not read current dir, probably no permissions.");
    return None;
  };
  loop {
    for filename in &filenames {
      let path = current_dir.join(filename);
      trace!("Current path: {:?}", path.to_str());
      if path.exists() {
        info!("Found {}", path.to_str().unwrap());
        return Some(path);
      }
    }
    if !current_dir.pop() {
      return None;
    }
  }
}

pub fn find_tailwind_file() -> Option<PathBuf> {
  let Ok(current_dir) = current_dir() else {
    error!("Could not read current dir, probably no permissions.");
    return None;
  };
  let base_path = current_dir.as_path();

  let tailwind_regex =
    RegexSet::new([r#"@import ["']tailwindcss["'];"#, r#"@tailwind base;"#]).unwrap();

  for result in Walk::new(base_path).filter_map(|e| e.ok()) {
    if result.file_type().filter(|file| file.is_file()).is_none() {
      continue;
    }

    let path = result.path();

    if !path.exists()
      || path
        .extension()
        .filter(|extension| *extension == "css")
        .is_none()
    {
      continue;
    }

    let contents = match read_to_string(path) {
      Ok(output) => output,
      Err(_) => continue,
    };

    if tailwind_regex.is_match(&contents) {
      let stripped_path = path.strip_prefix(base_path).unwrap();

      info!(
        "stripped tailwind path: {}",
        stripped_path.to_str().unwrap()
      );

      return Some(PathBuf::from(stripped_path));
    }
  }

  None
}
