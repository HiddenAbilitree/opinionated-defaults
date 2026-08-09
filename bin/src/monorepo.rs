use {
  crate::types::{PackageJSON, Packages},
  anyhow::{Context, Result},
  ignore::{WalkBuilder, overrides::OverrideBuilder},
  serde::Deserialize,
  std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    fs::{read_to_string, remove_file},
    io::ErrorKind,
    path::{Path, PathBuf},
  },
};

#[derive(Debug)]
pub struct WorkspaceProject {
  pub path: PathBuf,
  pub packages: Packages,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum OxlintConfig {
  Next,
  React,
}

impl OxlintConfig {
  const fn import(self) -> &'static str {
    match self {
      Self::Next => "oxlintConfigNext",
      Self::React => "oxlintConfigReact",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum GeneratedConfig {
  Next,
  TanstackStart,
}

impl GeneratedConfig {
  const fn oxlint_import(self) -> &'static str {
    match self {
      Self::Next => "oxlintConfigNext",
      Self::TanstackStart => "oxlintConfigTanstackStart",
    }
  }

  const fn oxfmt_import(self) -> &'static str {
    match self {
      Self::Next => "oxfmtConfigNext",
      Self::TanstackStart => "oxfmtConfigTanstackStart",
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct OxlintOverride {
  plugins: Vec<OxlintConfig>,
  rules: Vec<OxlintConfig>,
}

#[derive(Deserialize)]
struct PnpmWorkspace {
  #[serde(default)]
  packages: Vec<String>,
}

const CHILD_OX_CONFIG_FILENAMES: &[&str] = &["oxlint.config.ts", "oxfmt.config.ts"];
const NO_OXLINT_CONFIGS: &[OxlintConfig] = &[];
const NEXT_OXLINT_CONFIGS: &[OxlintConfig] = &[OxlintConfig::React, OxlintConfig::Next];
const REACT_OXLINT_CONFIGS: &[OxlintConfig] = &[OxlintConfig::React];

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

fn oxlint_configs(packages: &Packages) -> &'static [OxlintConfig] {
  if packages.contains_key("next") {
    NEXT_OXLINT_CONFIGS
  } else if packages.contains_key("react") || packages.contains_key("@tanstack/react-start") {
    REACT_OXLINT_CONFIGS
  } else {
    NO_OXLINT_CONFIGS
  }
}

fn insert_generated_configs(configs: &mut BTreeSet<GeneratedConfig>, packages: &Packages) {
  if packages.contains_key("next") {
    configs.insert(GeneratedConfig::Next);
  }
  if packages.contains_key("@tanstack/react-start")
    || packages.contains_key("@tanstack/solid-start")
  {
    configs.insert(GeneratedConfig::TanstackStart);
  }
}

fn workspace_generated_configs(
  packages: &Packages,
  projects: &[WorkspaceProject],
) -> BTreeSet<GeneratedConfig> {
  let mut configs = BTreeSet::new();
  insert_generated_configs(&mut configs, packages);
  for project in projects {
    insert_generated_configs(&mut configs, &project.packages);
  }
  configs
}

fn project_path(project: &WorkspaceProject) -> String {
  project.path.to_string_lossy().replace('\\', "/")
}

fn common_oxlint_configs(packages: &Packages, projects: &[WorkspaceProject]) -> Vec<OxlintConfig> {
  let Some(first) = projects.first() else {
    return oxlint_configs(packages).to_vec();
  };

  oxlint_configs(&first.packages)
    .iter()
    .copied()
    .filter(|config| {
      projects
        .iter()
        .all(|project| oxlint_configs(&project.packages).contains(config))
    })
    .collect()
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

pub fn build_oxlint_config(packages: &Packages, projects: &[WorkspaceProject]) -> String {
  let common_configs = common_oxlint_configs(packages, projects);
  let generated_configs = workspace_generated_configs(packages, projects);
  let mut override_groups: BTreeMap<OxlintOverride, Vec<String>> = BTreeMap::new();
  let mut imported_configs: BTreeSet<OxlintConfig> = common_configs.iter().copied().collect();

  for project in projects {
    let project_configs = oxlint_configs(&project.packages).to_vec();
    let difference_configs = project_configs
      .iter()
      .copied()
      .filter(|config| !common_configs.contains(config))
      .collect::<Vec<_>>();
    imported_configs.extend(project_configs.iter().copied());

    if difference_configs.is_empty() {
      continue;
    }

    override_groups
      .entry(OxlintOverride {
        plugins: project_configs,
        rules: difference_configs,
      })
      .or_default()
      .push(format!("{}/**/*", project_path(project)));
  }

  let next_roots = projects
    .iter()
    .filter(|project| project.packages.contains_key("next"))
    .map(|project| format!("{}/", project_path(project)))
    .collect::<Vec<_>>();
  let mut config = String::new();

  writeln!(config, "import {{").unwrap();
  writeln!(config, "  oxlintConfig,").unwrap();
  writeln!(config, "  oxlintConfigBase,").unwrap();
  for imported in &imported_configs {
    writeln!(config, "  {},", imported.import()).unwrap();
  }
  for generated in &generated_configs {
    if !imported_configs
      .iter()
      .any(|imported| imported.import() == generated.oxlint_import())
    {
      writeln!(config, "  {},", generated.oxlint_import()).unwrap();
    }
  }
  writeln!(
    config,
    "}} from '@hiddenability/opinionated-defaults/oxlint';\n"
  )
  .unwrap();
  writeln!(config, "export default oxlintConfig(").unwrap();
  writeln!(config, "  oxlintConfigBase,").unwrap();
  for common in &common_configs {
    writeln!(config, "  {},", common.import()).unwrap();
  }

  if !generated_configs.is_empty() || !override_groups.is_empty() || !next_roots.is_empty() {
    writeln!(config, "  {{").unwrap();
    if !generated_configs.is_empty() {
      writeln!(config, "    ignorePatterns: [").unwrap();
      for generated in &generated_configs {
        writeln!(
          config,
          "      ...({}.ignorePatterns ?? []),",
          generated.oxlint_import()
        )
        .unwrap();
      }
      writeln!(config, "    ],").unwrap();
    }
    if !override_groups.is_empty() {
      writeln!(config, "    overrides: [").unwrap();
      for (override_config, files) in override_groups {
        writeln!(config, "      {{").unwrap();
        writeln!(
          config,
          "        files: {},",
          serde_json::to_string(&files).expect("override paths should serialize")
        )
        .unwrap();
        writeln!(config, "        plugins: [").unwrap();
        writeln!(config, "          ...(oxlintConfigBase.plugins ?? []),").unwrap();
        for plugin_config in override_config.plugins {
          writeln!(
            config,
            "          ...({}.plugins ?? []),",
            plugin_config.import()
          )
          .unwrap();
        }
        writeln!(config, "        ],").unwrap();
        writeln!(config, "        rules: {{").unwrap();
        for rule_config in override_config.rules {
          writeln!(
            config,
            "          ...({}.rules ?? {{}}),",
            rule_config.import()
          )
          .unwrap();
        }
        writeln!(config, "        }},").unwrap();
        writeln!(config, "      }},").unwrap();
      }
      writeln!(config, "    ],").unwrap();
    }
    if !next_roots.is_empty() {
      writeln!(config, "    settings: {{").unwrap();
      writeln!(config, "      next: {{").unwrap();
      writeln!(
        config,
        "        rootDir: {},",
        serde_json::to_string(&next_roots).expect("Next.js root paths should serialize")
      )
      .unwrap();
      writeln!(config, "      }},").unwrap();
      writeln!(config, "    }},").unwrap();
    }
    writeln!(config, "  }},").unwrap();
  }

  writeln!(config, ");").unwrap();
  config
}

pub fn build_oxfmt_config(packages: &Packages, projects: &[WorkspaceProject]) -> String {
  let generated_configs = workspace_generated_configs(packages, projects);
  let mut config = String::new();

  writeln!(config, "import {{").unwrap();
  writeln!(config, "  oxfmtConfig,").unwrap();
  writeln!(config, "  oxfmtConfigBase,").unwrap();
  for generated in &generated_configs {
    writeln!(config, "  {},", generated.oxfmt_import()).unwrap();
  }
  writeln!(
    config,
    "}} from '@hiddenability/opinionated-defaults/oxfmt';\n"
  )
  .unwrap();
  writeln!(config, "export default oxfmtConfig(").unwrap();
  writeln!(config, "  oxfmtConfigBase,").unwrap();
  for generated in &generated_configs {
    writeln!(config, "  {},", generated.oxfmt_import()).unwrap();
  }
  writeln!(config, ");").unwrap();
  config
}

pub fn remove_child_ox_configs(root: &Path, projects: &[WorkspaceProject]) -> Result<()> {
  for project in projects {
    for filename in CHILD_OX_CONFIG_FILENAMES {
      let path = root.join(&project.path).join(filename);
      if let Err(error) = remove_file(&path)
        && error.kind() != ErrorKind::NotFound
      {
        return Err(error).with_context(|| format!("could not remove {}", path.display()));
      }
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    serde_json::Value,
    std::{
      fs::{create_dir_all, remove_dir_all, write},
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

  fn packages(names: &[&str]) -> Packages {
    names
      .iter()
      .map(|name| ((*name).into(), Value::String("latest".into())))
      .collect()
  }

  fn project(path: &str, package_names: &[&str]) -> WorkspaceProject {
    WorkspaceProject {
      path: PathBuf::from(path),
      packages: packages(package_names),
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

  #[test]
  fn renders_project_differences_from_imported_npm_configs() {
    let projects = [
      project("apps/api", &["elysia"]),
      project("apps/admin", &["react"]),
      project("apps/nexus", &["next"]),
      project("apps/web", &["react"]),
    ];

    let config = build_oxlint_config(&packages(&["turbo"]), &projects);

    assert!(config.contains("oxlintConfigBase"));
    assert!(config.contains("oxlintConfigReact"));
    assert!(config.contains("oxlintConfigNext"));
    assert!(config.contains(r#"files: ["apps/admin/**/*","apps/web/**/*"]"#));
    assert!(config.contains("...(oxlintConfigNext.rules ?? {})"));
    assert!(config.contains("...(oxlintConfigNext.ignorePatterns ?? [])"));
    assert!(config.contains(r#"rootDir: ["apps/nexus/"]"#));
  }

  #[test]
  fn promotes_framework_config_shared_by_every_project() {
    let projects = [
      project("apps/admin", &["react"]),
      project("apps/start", &["@tanstack/react-start"]),
    ];

    let config = build_oxlint_config(&packages(&["turbo"]), &projects);

    assert!(config.contains("  oxlintConfigReact,\n  {"));
    assert!(config.contains("...(oxlintConfigTanstackStart.ignorePatterns ?? [])"));
    assert!(!config.contains("overrides:"));
  }

  #[test]
  fn renders_formatter_from_detected_npm_config_fragments() {
    let projects = [
      project("apps/nexus", &["next"]),
      project("apps/start", &["@tanstack/solid-start"]),
    ];

    let config = build_oxfmt_config(&packages(&["turbo"]), &projects);

    assert!(config.contains("oxfmtConfigBase"));
    assert!(config.contains("oxfmtConfigNext"));
    assert!(config.contains("oxfmtConfigTanstackStart"));
  }
}
