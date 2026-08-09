use {
  crate::{
    get_package_manager::find_workspace_projects,
    handle_dependencies::handle_dependencies,
    types::{Dependencies, Packages, TSConfig, Tooling, WorkspaceProject},
    utils::{find_file, find_files, find_tailwind_file},
  },
  anyhow::{Context, Result},
  jsonc_parser::{ParseOptions, parse_to_serde_value},
  pathdiff::diff_paths,
  serde_json::{Map, Value, from_str, from_value, json, to_string_pretty},
  std::{
    env::current_dir,
    fmt::Write,
    fs::{read_to_string, remove_file, write},
    io::ErrorKind,
    path::{Path, PathBuf},
  },
};

fn dep(pkg: &str, import: &str) -> (String, String) {
  (pkg.into(), import.into())
}

#[derive(Clone, Copy)]
struct DefaultProjectConfig<'a> {
  files: &'a [&'a str],
  default_project: Option<&'a str>,
}

struct TsconfigEslintConfig {
  files: Vec<&'static str>,
  project: Option<&'static str>,
  uses_paths: bool,
}

const GENERATED_TS_IGNORE_PATTERN: &str = "**/*.gen.ts";

fn build_eslint_config(
  imports: &[String],
  gitignore_paths: &[&Path],
  default_project_config: DefaultProjectConfig,
) -> String {
  let mut out = String::new();

  if !gitignore_paths.is_empty() {
    writeln!(out, "import {{ includeIgnoreFile }} from '@eslint/compat';").unwrap();
  }

  writeln!(out, "import {{").unwrap();
  writeln!(out, "  eslintConfig,").unwrap();
  for import in imports {
    writeln!(out, "  {import},").unwrap();
  }
  writeln!(out, "}} from '@hiddenability/opinionated-defaults/eslint';").unwrap();
  writeln!(out, "import {{ fileURLToPath }} from 'node:url';").unwrap();
  writeln!(out).unwrap();

  writeln!(out, "export default eslintConfig([").unwrap();
  for path in gitignore_paths {
    writeln!(
      out,
      "  includeIgnoreFile(fileURLToPath(new URL(`{}`, import.meta.url)), ``),",
      path.display()
    )
    .unwrap();
  }
  for import in imports {
    if import == "eslintConfigDefaultProject" {
      let files_str = default_project_config
        .files
        .iter()
        .map(|f| format!("`{f}`"))
        .collect::<Vec<_>>()
        .join(", ");

      match default_project_config.default_project {
        Some(default_project) => {
          writeln!(out, "  ...{import}({{").unwrap();
          writeln!(out, "    allowDefaultProject: [{files_str}],").unwrap();
          writeln!(out, "    defaultProject: `{default_project}`,").unwrap();
          writeln!(out, "  }}),").unwrap();
        }
        None => {
          writeln!(out, "  ...{import}([{files_str}]),").unwrap();
        }
      }
    } else {
      writeln!(out, "  ...{import},").unwrap();
    }
  }
  writeln!(out, "]);").unwrap();

  out
}

fn build_prettier_config(imports: &[String], tailwind_path: Option<&Path>) -> String {
  let mut out = String::new();

  writeln!(out, "import {{").unwrap();
  writeln!(out, "  prettierConfig,").unwrap();
  for import in imports {
    writeln!(out, "  {import},").unwrap();
  }
  writeln!(
    out,
    "}} from '@hiddenability/opinionated-defaults/prettier';"
  )
  .unwrap();
  writeln!(out).unwrap();

  write!(out, "export default prettierConfig(").unwrap();
  for (i, import) in imports.iter().enumerate() {
    if i > 0 {
      write!(out, ", ").unwrap();
    }
    write!(out, "{import}").unwrap();
  }
  if let Some(path) = tailwind_path {
    write!(
      out,
      ", {{\n  tailwindStylesheet: `./{}`,\n}}",
      path.display()
    )
    .unwrap();
  }
  writeln!(out, ");").unwrap();

  out
}

fn ensure_ignore_pattern(path: &str, pattern: &str) -> Result<()> {
  let mut contents = match read_to_string(path) {
    Ok(contents) => contents,
    Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
    Err(error) => return Err(error.into()),
  };

  if contents.lines().any(|line| line.trim() == pattern) {
    return Ok(());
  }

  if !contents.is_empty() && !contents.ends_with('\n') {
    contents.push('\n');
  }

  contents.push_str(pattern);
  contents.push('\n');

  write(path, contents)?;

  Ok(())
}

fn has_package(packages: &Packages, name: &str) -> bool {
  packages.contains_key(name)
}

fn push_plugin(plugins: &mut Vec<Value>, plugin: &str) {
  if plugins.iter().any(|value| value.as_str() == Some(plugin)) {
    return;
  }

  plugins.push(Value::String(plugin.into()));
}

fn add_framework_plugins(plugins: &mut Vec<Value>, packages: &Packages) {
  let has_next = has_package(packages, "next");
  let has_react = has_next || has_package(packages, "react");

  if has_react {
    push_plugin(plugins, "react");
    push_plugin(plugins, "react-perf");
  }

  if has_next {
    push_plugin(plugins, "nextjs");
  }
}

const REACT_CORRECTNESS_RULES: &[&str] = &[
  "exhaustive-deps",
  "forward-ref-uses-ref",
  "jsx-key",
  "jsx-no-duplicate-props",
  "jsx-no-undef",
  "jsx-props-no-spread-multi",
  "no-children-prop",
  "no-danger-with-children",
  "no-did-mount-set-state",
  "no-did-update-set-state",
  "no-direct-mutation-state",
  "no-find-dom-node",
  "no-is-mounted",
  "no-render-return-value",
  "no-string-refs",
  "no-this-in-sfc",
  "no-unsafe",
  "no-will-update-set-state",
  "void-dom-elements-no-children",
];
const REACT_SUSPICIOUS_RULES: &[&str] = &[
  "iframe-missing-sandbox",
  "jsx-no-comment-textnodes",
  "jsx-no-script-url",
  "no-namespace",
  "no-unstable-nested-components",
  "style-prop-object",
];
const NEXT_CORRECTNESS_RULES: &[&str] = &[
  "google-font-display",
  "google-font-preconnect",
  "inline-script-id",
  "next-script-for-ga",
  "no-assign-module-variable",
  "no-async-client-component",
  "no-before-interactive-script-outside-document",
  "no-css-tags",
  "no-document-import-in-page",
  "no-duplicate-head",
  "no-head-element",
  "no-head-import-in-document",
  "no-html-link-for-pages",
  "no-img-element",
  "no-page-custom-font",
  "no-script-component-in-head",
  "no-styled-jsx-in-document",
  "no-sync-scripts",
  "no-title-in-document-head",
  "no-typos",
  "no-unwanted-polyfillio",
];

fn build_oxlint_config_value(packages: &Packages) -> Value {
  let mut config: Value = from_str(include_str!(concat!(env!("OUT_DIR"), "/oxlintrc.json")))
    .expect("embedded JSON should be valid");

  if let Some(obj) = config.as_object_mut() {
    obj.remove("$schema");

    if let Some(plugins) = obj.get_mut("plugins").and_then(Value::as_array_mut) {
      add_framework_plugins(plugins, packages);
    }
  }

  config
}

fn project_path(project: &WorkspaceProject) -> String {
  project.path.to_string_lossy().replace('\\', "/")
}

fn project_glob(project: &WorkspaceProject) -> String {
  format!("{}/**/*", project_path(project))
}

fn insert_plugin_rules(
  rules: &mut Map<String, Value>,
  plugin: &str,
  names: &[&str],
  severity: &Value,
) {
  if severity.as_str() == Some("off") || severity.as_u64() == Some(0) {
    return;
  }

  for name in names {
    rules.insert(format!("{plugin}/{name}"), severity.clone());
  }
}

fn build_framework_rules(
  packages: &Packages,
  correctness: &Value,
  suspicious: &Value,
) -> Map<String, Value> {
  let mut rules = Map::new();
  let has_next = has_package(packages, "next");
  if has_next || has_package(packages, "react") {
    insert_plugin_rules(&mut rules, "react", REACT_CORRECTNESS_RULES, correctness);
    insert_plugin_rules(&mut rules, "react", REACT_SUSPICIOUS_RULES, suspicious);
  }
  if has_next {
    insert_plugin_rules(&mut rules, "nextjs", NEXT_CORRECTNESS_RULES, correctness);
  }
  rules
}

fn build_monorepo_oxlint_config_value(packages: &Packages, projects: &[WorkspaceProject]) -> Value {
  if projects.is_empty() {
    return build_oxlint_config_value(packages);
  }

  // Oxlint categories are root-only. Keep the root framework-neutral, then
  // mirror category-enabled framework rules into each matching override.
  let mut config = build_oxlint_config_value(&Packages::new());
  let correctness = config
    .pointer("/categories/correctness")
    .cloned()
    .unwrap_or_else(|| Value::String("off".into()));
  let suspicious = config
    .pointer("/categories/suspicious")
    .cloned()
    .unwrap_or_else(|| Value::String("off".into()));
  let overrides = projects
    .iter()
    .map(|project| {
      let plugins = build_oxlint_config_value(&project.packages)
        .get("plugins")
        .cloned()
        .expect("embedded Oxlint config should contain plugins");
      let rules = build_framework_rules(&project.packages, &correctness, &suspicious);
      let mut override_config = json!({
        "files": [project_glob(project)],
        "plugins": plugins,
      });
      if !rules.is_empty() {
        override_config
          .as_object_mut()
          .expect("Oxlint override should be an object")
          .insert("rules".into(), Value::Object(rules));
      }
      override_config
    })
    .collect();
  let next_roots: Vec<_> = projects
    .iter()
    .filter(|project| has_package(&project.packages, "next"))
    .map(|project| Value::String(format!("{}/", project_path(project))))
    .collect();

  if let Some(obj) = config.as_object_mut() {
    obj.insert("overrides".into(), Value::Array(overrides));

    if !next_roots.is_empty() {
      let settings = obj
        .entry("settings")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("Oxlint settings should be an object");
      let next = settings
        .entry("next")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("Oxlint Next.js settings should be an object");
      next.insert("rootDir".into(), Value::Array(next_roots));
    }
  }

  config
}

fn build_oxlint_config(packages: &Packages, projects: &[WorkspaceProject]) -> String {
  let config = to_string_pretty(&build_monorepo_oxlint_config_value(packages, projects))
    .expect("serialization should succeed");
  format!("import {{ defineConfig }} from 'oxlint';\n\nexport default defineConfig({config});\n")
}

fn build_oxfmt_config_value() -> Value {
  let mut config: Value = from_str(include_str!(concat!(env!("OUT_DIR"), "/oxfmtrc.json")))
    .expect("embedded JSON should be valid");
  if let Some(obj) = config.as_object_mut() {
    obj.remove("$schema");
  }
  config
}

fn build_monorepo_oxfmt_config_value(projects: &[WorkspaceProject]) -> Value {
  let mut config = build_oxfmt_config_value();
  if projects.is_empty() {
    return config;
  }

  let mut project_options = config.clone();
  if let Some(options) = project_options.as_object_mut() {
    options.remove("ignorePatterns");
    options.remove("overrides");
  }
  let overrides = projects
    .iter()
    .map(|project| {
      json!({
        "files": [project_glob(project)],
        "options": project_options,
      })
    })
    .collect();

  config
    .as_object_mut()
    .expect("embedded Oxfmt config should be an object")
    .insert("overrides".into(), Value::Array(overrides));
  config
}

fn build_oxfmt_config(projects: &[WorkspaceProject]) -> String {
  let config = to_string_pretty(&build_monorepo_oxfmt_config_value(projects))
    .expect("serialization should succeed");
  format!("import {{ defineConfig }} from 'oxfmt';\n\nexport default defineConfig({config});\n")
}

fn find_default_project_tsconfig(tsconfig: &TSConfig) -> Option<&'static str> {
  const CANDIDATES: &[&str] = &["tsconfig.node.json", "tsconfig.eslint.json"];

  tsconfig.is_solution_style().then(|| {
    CANDIDATES
      .iter()
      .find(|name| find_file(name).is_some())
      .copied()
      .unwrap_or("tsconfig.node.json")
  })
}

fn is_ts_include_pattern(pattern: &str) -> bool {
  pattern == "."
    || pattern == "*"
    || pattern == "*.ts"
    || pattern.contains("**/*.ts")
    || pattern == "eslint.config.ts"
}

fn is_mjs_include_pattern(pattern: &str) -> bool {
  pattern == "."
    || pattern == "*"
    || pattern == "*.mjs"
    || pattern.contains("**/*.mjs")
    || pattern == "prettier.config.mjs"
}

fn has_path_aliases(tsconfig: &TSConfig) -> bool {
  tsconfig
    .compiler_options
    .as_ref()
    .and_then(|opts| opts.paths.as_ref())
    .is_some_and(|paths| !paths.is_empty())
}

fn included_by_tsconfig(tsconfig: &TSConfig, matches: fn(&str) -> bool) -> bool {
  tsconfig
    .include
    .as_ref()
    .is_none_or(|includes| includes.iter().any(|pattern| matches(pattern)))
}

fn includes_mjs(tsconfig: &TSConfig) -> bool {
  let allow_js = tsconfig
    .compiler_options
    .as_ref()
    .is_some_and(|opts| opts.allow_js);

  (allow_js && tsconfig.include.is_none())
    || tsconfig.include.as_ref().is_some_and(|includes| {
      includes
        .iter()
        .any(|pattern| is_mjs_include_pattern(pattern))
    })
}

fn build_tsconfig_eslint_config(tsconfig: &TSConfig) -> TsconfigEslintConfig {
  let mut files = Vec::new();
  let has_empty_files = tsconfig.files.as_ref().is_some_and(Vec::is_empty);
  let includes_ts = included_by_tsconfig(tsconfig, is_ts_include_pattern);
  let includes_mjs = includes_mjs(tsconfig);

  if has_empty_files || !includes_ts {
    files.push("eslint.config.ts");
  }

  if has_empty_files || !includes_mjs {
    files.push("prettier.config.mjs");
  }

  if !includes_mjs && find_file("postcss.config.mjs").is_some() {
    files.push("postcss.config.mjs");
  }

  TsconfigEslintConfig {
    files,
    project: find_default_project_tsconfig(tsconfig),
    uses_paths: has_path_aliases(tsconfig),
  }
}

fn read_tsconfig_eslint_config() -> Result<Option<TsconfigEslintConfig>> {
  let Some(tsconfig_path) = find_file("tsconfig.json") else {
    return Ok(None);
  };

  let contents = read_to_string(tsconfig_path)?;
  let parsed = parse_to_serde_value(&contents, &ParseOptions::default())?
    .and_then(|value| from_value::<TSConfig>(value).ok());

  parsed.map_or_else(
    || {
      eprintln!("❌ Could not parse tsconfig.json. Skipping relative check...");
      Ok(None)
    },
    |config| Ok(Some(build_tsconfig_eslint_config(&config))),
  )
}

fn find_configured_tailwind_path(prettier_imports: &[String]) -> Option<PathBuf> {
  if !prettier_imports
    .iter()
    .any(|s| s == "prettierConfigTailwind")
  {
    return None;
  }

  find_tailwind_file().map_or_else(
    || {
      eprintln!(
        "⚠️ TailwindCSS dependency found but could not find a relevant css file. Skipping..."
      );
      None
    },
    Some,
  )
}

fn update_scripts(scripts: &[(&str, &str)]) -> Result<()> {
  let path = "package.json";
  if let Ok(contents) = read_to_string(path) {
    let mut v: Value = from_str(&contents)?;

    if let Some(map) = v.get_mut("scripts").and_then(|s| s.as_object_mut()) {
      for &(key, value) in scripts {
        map.insert(key.into(), Value::String(value.into()));
      }
    } else {
      let mut map = Map::new();
      for &(key, value) in scripts {
        map.insert(key.into(), Value::String(value.into()));
      }

      if let Some(obj) = v.as_object_mut() {
        obj.insert("scripts".into(), Value::Object(map));
      }
    }

    let new_contents = to_string_pretty(&v)? + "\n";
    write(path, new_contents)?;
  }
  Ok(())
}

fn generate_eslint_config(packages: Packages) -> Result<()> {
  let mut eslint_imports = handle_dependencies(Dependencies {
    packages: packages.clone(),
    valid_deps: vec![
      dep("astro", "eslintConfigAstro"),
      dep("react", "eslintConfigReact"),
      dep("solid-js", "eslintConfigSolid"),
      dep("turborepo", "eslintConfigTurbo"),
      dep("next", "eslintConfigNext"),
    ],
    default_deps: vec![
      "eslintConfigBase".into(),
      "eslintConfigPerfectionist".into(),
      "eslintConfigPrettier".into(),
    ],
  });

  let mut default_project_files = Vec::new();
  let mut default_project_tsconfig: Option<&str> = None;

  if let Some(config) = read_tsconfig_eslint_config()? {
    if config.uses_paths {
      eslint_imports.push("eslintConfigRelative".into());
    }
    default_project_tsconfig = config.project;
    default_project_files = config.files;

    if !default_project_files.is_empty() {
      eslint_imports.push("eslintConfigDefaultProject".into());
    }
  }

  eslint_imports.sort();

  let cwd = current_dir()?;
  let gitignore_paths: Vec<_> = find_files(".gitignore")
    .into_iter()
    .filter_map(|path| diff_paths(&path, &cwd))
    .collect();
  let gitignore_refs: Vec<_> = gitignore_paths.iter().map(PathBuf::as_path).collect();

  let eslint_config = build_eslint_config(
    &eslint_imports,
    &gitignore_refs,
    DefaultProjectConfig {
      files: &default_project_files,
      default_project: default_project_tsconfig,
    },
  );

  let prettier_imports = handle_dependencies(Dependencies {
    packages,
    valid_deps: vec![dep("tailwindcss", "prettierConfigTailwind")],
    default_deps: vec!["prettierConfigBase".into()],
  });

  let tailwind_path = find_configured_tailwind_path(&prettier_imports);
  let prettier_config = build_prettier_config(&prettier_imports, tailwind_path.as_deref());

  write("eslint.config.ts", eslint_config)?;
  write("prettier.config.mjs", prettier_config)?;
  ensure_ignore_pattern(".gitignore", GENERATED_TS_IGNORE_PATTERN)?;

  update_scripts(&[("lint", "eslint ."), ("lint:fix", "eslint . --fix")])?;

  Ok(())
}

const CHILD_OX_CONFIG_FILENAMES: &[&str] = &[
  "oxlint.config.ts",
  ".oxlintrc.json",
  ".oxlintrc.jsonc",
  "oxfmt.config.ts",
  ".oxfmtrc.json",
  ".oxfmtrc.jsonc",
];

fn remove_child_ox_configs(root: &Path, projects: &[WorkspaceProject]) -> Result<()> {
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

fn generate_ox_config(packages: Packages) -> Result<()> {
  let root = current_dir()?;
  let projects = find_workspace_projects(&root)?;

  write(
    root.join("oxlint.config.ts"),
    build_oxlint_config(&packages, &projects),
  )?;
  write(root.join("oxfmt.config.ts"), build_oxfmt_config(&projects))?;
  remove_child_ox_configs(&root, &projects)?;

  update_scripts(&[
    ("lint", "oxlint"),
    ("lint:fix", "oxlint --fix"),
    ("format", "oxfmt ."),
    ("format:check", "oxfmt --check ."),
  ])?;

  Ok(())
}

pub fn generate_config(packages: Packages, tooling: Tooling) -> Result<()> {
  match tooling {
    Tooling::Eslint => generate_eslint_config(packages),
    Tooling::Ox => generate_ox_config(packages),
  }
}

#[cfg(test)]
mod tests {
  use {super::*, serde_json::Value};

  fn packages(names: &[&str]) -> Packages {
    names
      .iter()
      .map(|name| ((*name).into(), Value::String("latest".into())))
      .collect()
  }

  fn plugin_names(config: &Value) -> Vec<&str> {
    config
      .get("plugins")
      .and_then(Value::as_array)
      .expect("plugins should be an array")
      .iter()
      .map(|plugin| plugin.as_str().expect("plugin names should be strings"))
      .collect()
  }

  fn workspace_project(path: &str, package_names: &[&str]) -> WorkspaceProject {
    WorkspaceProject {
      path: PathBuf::from(path),
      packages: packages(package_names),
    }
  }

  fn override_for<'a>(config: &'a Value, glob: &str) -> &'a Value {
    config
      .get("overrides")
      .and_then(Value::as_array)
      .expect("overrides should be an array")
      .iter()
      .find(|override_config| {
        override_config
          .get("files")
          .and_then(Value::as_array)
          .is_some_and(|files| files.first().and_then(Value::as_str) == Some(glob))
      })
      .expect("project override should exist")
  }

  #[test]
  fn oxlint_config_stays_framework_neutral_by_default() {
    let config = build_oxlint_config_value(&packages(&[]));
    let plugins = plugin_names(&config);

    assert!(!plugins.contains(&"react"));
    assert!(!plugins.contains(&"react-perf"));
    assert!(!plugins.contains(&"nextjs"));
  }

  #[test]
  fn tanstack_react_start_oxlint_config_gets_react_without_nextjs() {
    let config =
      build_oxlint_config_value(&packages(&["@tanstack/react-start", "react", "react-dom"]));
    let plugins = plugin_names(&config);

    assert!(plugins.contains(&"react"));
    assert!(plugins.contains(&"react-perf"));
    assert!(!plugins.contains(&"nextjs"));
  }

  #[test]
  fn tanstack_solid_start_oxlint_config_gets_no_react_or_nextjs_plugins() {
    let config = build_oxlint_config_value(&packages(&["@tanstack/solid-start", "solid-js"]));
    let plugins = plugin_names(&config);

    assert!(!plugins.contains(&"react"));
    assert!(!plugins.contains(&"react-perf"));
    assert!(!plugins.contains(&"nextjs"));
  }

  #[test]
  fn next_projects_get_nextjs_and_react_oxlint_plugins() {
    let config = build_oxlint_config_value(&packages(&["next"]));
    let plugins = plugin_names(&config);

    assert!(plugins.contains(&"react"));
    assert!(plugins.contains(&"react-perf"));
    assert!(plugins.contains(&"nextjs"));
  }

  #[test]
  fn monorepo_oxlint_overrides_scope_plugins_by_project_type() {
    let projects = [
      workspace_project("apps/api", &["elysia"]),
      workspace_project("apps/nexus", &["next", "react"]),
      workspace_project("packages/db", &["drizzle-orm"]),
    ];

    let config = build_monorepo_oxlint_config_value(&packages(&["turbo"]), &projects);
    let api_override = override_for(&config, "apps/api/**/*");
    let next_override = override_for(&config, "apps/nexus/**/*");
    let api_plugins = plugin_names(api_override);
    let next_plugins = plugin_names(next_override);

    assert!(api_plugins.contains(&"node"));
    assert!(!api_plugins.contains(&"react"));
    assert!(!api_plugins.contains(&"nextjs"));
    assert!(api_override.get("rules").is_none());
    assert!(next_plugins.contains(&"react"));
    assert!(next_plugins.contains(&"react-perf"));
    assert!(next_plugins.contains(&"nextjs"));
    assert_eq!(
      next_override
        .get("rules")
        .and_then(|rules| rules.get("nextjs/no-img-element")),
      Some(&json!("error"))
    );
    assert_eq!(
      next_override
        .get("rules")
        .and_then(|rules| rules.get("react/no-unstable-nested-components")),
      Some(&json!("warn"))
    );
    assert_eq!(
      config
        .pointer("/settings/next/rootDir")
        .expect("Next.js root directories should be configured"),
      &json!(["apps/nexus/"])
    );
  }

  #[test]
  fn monorepo_oxfmt_config_has_an_override_for_every_project() {
    let projects = [
      workspace_project("apps/api", &["elysia"]),
      workspace_project("apps/nexus", &["next"]),
    ];

    let config = build_monorepo_oxfmt_config_value(&projects);
    for glob in ["apps/api/**/*", "apps/nexus/**/*"] {
      let options = override_for(&config, glob)
        .get("options")
        .expect("project override should contain format options");
      assert!(options.get("sortImports").is_some());
      assert!(options.get("sortTailwindcss").is_some());
      assert!(options.get("ignorePatterns").is_none());
    }
  }
}
