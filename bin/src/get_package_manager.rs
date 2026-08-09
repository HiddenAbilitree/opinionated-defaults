use {
  crate::{
    types::{PackageJSON, PackageManager, Packages, ProjectData, WorkspaceProject},
    utils::{find_file, find_first_file},
  },
  anyhow::{Context, Result},
  ignore::{WalkBuilder, overrides::OverrideBuilder},
  log::info,
  serde::Deserialize,
  std::{fs::read_to_string, io::ErrorKind, path::Path},
};

fn read_package_json_packages() -> Option<Packages> {
  let path = find_file("package.json")?;
  let content = read_to_string(&path).ok()?;
  let data: PackageJSON = serde_json::from_str(&content).ok()?;

  Some(data.into_packages())
}

#[derive(Deserialize)]
struct PnpmWorkspace {
  #[serde(default)]
  packages: Vec<String>,
}

fn read_workspace_patterns(root: &Path) -> Result<Vec<String>> {
  let package_json_path = root.join("package.json");
  let contents = match read_to_string(&package_json_path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => {
      return Err(error).with_context(|| format!("could not read {}", package_json_path.display()));
    }
  };
  let package_json: PackageJSON = serde_json::from_str(&contents)
    .with_context(|| format!("could not parse {}", package_json_path.display()))?;

  if let Some(workspaces) = package_json.workspaces {
    return Ok(workspaces.patterns().to_vec());
  }

  let pnpm_workspace_path = root.join("pnpm-workspace.yaml");
  let contents = match read_to_string(&pnpm_workspace_path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
    Err(error) => {
      return Err(error)
        .with_context(|| format!("could not read {}", pnpm_workspace_path.display()));
    }
  };
  let workspace: PnpmWorkspace = serde_yaml::from_str(&contents)
    .with_context(|| format!("could not parse {}", pnpm_workspace_path.display()))?;

  Ok(workspace.packages)
}

fn workspace_package_glob(pattern: &str) -> Option<String> {
  let pattern = pattern.trim();
  if pattern.is_empty() {
    return None;
  }

  let (prefix, pattern) = pattern
    .strip_prefix('!')
    .map_or(("", pattern), |pattern| ("!", pattern));
  let pattern = pattern
    .strip_prefix("./")
    .unwrap_or(pattern)
    .trim_end_matches('/');

  (!pattern.is_empty()).then(|| format!("{prefix}{pattern}/package.json"))
}

pub fn find_workspace_projects(root: &Path) -> Result<Vec<WorkspaceProject>> {
  let patterns = read_workspace_patterns(root)?;
  if patterns.is_empty() {
    return Ok(Vec::new());
  }

  let mut matcher = OverrideBuilder::new(root);
  for pattern in &patterns {
    if let Some(glob) = workspace_package_glob(pattern) {
      matcher
        .add(&glob)
        .with_context(|| format!("invalid workspace pattern {pattern:?}"))?;
    }
  }
  let matcher = matcher
    .build()
    .context("could not build workspace matcher")?;

  let mut projects = Vec::new();
  for entry in WalkBuilder::new(root).build() {
    let entry = entry.context("could not traverse workspace")?;
    if !entry.file_type().is_some_and(|kind| kind.is_file())
      || entry.file_name() != "package.json"
      || !matcher.matched(entry.path(), false).is_whitelist()
    {
      continue;
    }

    let package_json_path = entry.path();
    let path = package_json_path
      .parent()
      .expect("package.json should have a parent")
      .strip_prefix(root)
      .context("workspace project should be inside its root")?
      .to_path_buf();
    let package_json: PackageJSON = serde_json::from_str(
      &read_to_string(package_json_path)
        .with_context(|| format!("could not read {}", package_json_path.display()))?,
    )
    .with_context(|| format!("could not parse {}", package_json_path.display()))?;

    projects.push(WorkspaceProject {
      path,
      packages: package_json.into_packages(),
    });
  }

  projects.sort_by(|left, right| left.path.cmp(&right.path));
  Ok(projects)
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

#[cfg(test)]
mod tests {
  use {
    super::*,
    std::{
      fs::{create_dir_all, remove_dir_all, write},
      path::PathBuf,
      process,
      time::{SystemTime, UNIX_EPOCH},
    },
  };

  struct TempWorkspace(PathBuf);

  impl TempWorkspace {
    fn new() -> Self {
      let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_nanos();
      let path = std::env::temp_dir().join(format!(
        "opinionated-defaults-workspace-{}-{nonce}",
        process::id()
      ));
      create_dir_all(&path).expect("temporary workspace should be created");
      Self(path)
    }
  }

  impl Drop for TempWorkspace {
    fn drop(&mut self) {
      let _ = remove_dir_all(&self.0);
    }
  }

  #[test]
  fn discovers_declared_projects_from_package_manifests_only() {
    let workspace = TempWorkspace::new();
    for directory in ["apps/api", "apps/nexus", "examples/demo"] {
      create_dir_all(workspace.0.join(directory))
        .expect("workspace project directory should be created");
    }

    write(
      workspace.0.join("package.json"),
      r#"{"workspaces":["apps/*"]}"#,
    )
    .expect("root package.json should be written");
    write(
      workspace.0.join("apps/api/package.json"),
      r#"{"dependencies":{"elysia":"latest"}}"#,
    )
    .expect("API package.json should be written");
    write(
      workspace.0.join("apps/api/oxlint.config.ts"),
      "export default { plugins: ['nextjs'] };",
    )
    .expect("existing config should be written");
    write(
      workspace.0.join("apps/nexus/package.json"),
      r#"{"dependencies":{"next":"latest"}}"#,
    )
    .expect("Next.js package.json should be written");
    write(
      workspace.0.join("examples/demo/package.json"),
      r#"{"dependencies":{"next":"latest"}}"#,
    )
    .expect("non-workspace package.json should be written");

    let projects =
      find_workspace_projects(&workspace.0).expect("workspace projects should be discovered");

    assert_eq!(
      projects
        .iter()
        .map(|project| project.path.clone())
        .collect::<Vec<_>>(),
      [PathBuf::from("apps/api"), PathBuf::from("apps/nexus")]
    );
    assert!(projects[0].packages.contains_key("elysia"));
    assert!(!projects[0].packages.contains_key("next"));
    assert!(projects[1].packages.contains_key("next"));
  }
}
