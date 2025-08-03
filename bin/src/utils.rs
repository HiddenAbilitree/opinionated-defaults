use {
  anyhow::{Result, anyhow},
  ignore::Walk,
  log::info,
  regex::RegexSet,
  std::{env::current_dir, fs::read_to_string, path::PathBuf},
};

// from https://github.com/lbwa/package-json-rs/blob/main/src/fs.rs#L10-L25
pub fn find_file(filename: &str) -> Option<PathBuf> {
  let mut current_dir = PathBuf::from(current_dir().as_ref().expect("probably no permissions"));
  loop {
    let path = current_dir.join(filename);
    if path.exists() {
      info!("Found {}", path.to_str().unwrap());
      return Some(path);
    }
    if !current_dir.pop() {
      return None;
    }
  }
}

pub fn find_tailwind_file() -> Result<PathBuf> {
  let binding = current_dir()?;
  let base_path = binding.as_path();

  let tailwind_regex = RegexSet::new([r#"@import ["']tailwindcss["'];"#, r#"@tailwind base;"#])?;

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
      let stripped_path = path.strip_prefix(base_path)?;

      info!(
        "stripped tailwind path: {}",
        stripped_path.to_str().unwrap()
      );

      return Ok(PathBuf::from(path.strip_prefix(base_path)?));
    }
  }
  Err(anyhow!("couldnt find a tailwind file"))
}
