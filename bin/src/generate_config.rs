use {
  crate::{
    handle_dependencies::handle_dependencies,
    types::{Dependencies, Packages, TSConfig, Tooling},
    utils::{find_file, find_files, find_tailwind_file},
  },
  anyhow::Result,
  jsonc_parser::{ParseOptions, parse_to_serde_value},
  pathdiff::diff_paths,
  serde_json::{Map, Value, from_str, from_value, to_string_pretty},
  std::{
    env::current_dir,
    fmt::Write,
    fs::{read_to_string, write},
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

fn strip_schema(json: &str) -> String {
  let mut config: Value = from_str(json).expect("embedded JSON should be valid");
  if let Some(obj) = config.as_object_mut() {
    obj.remove("$schema");
  }
  to_string_pretty(&config).expect("serialization should succeed")
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

fn build_oxlint_config_value(packages: &Packages) -> Value {
  let mut config: Value = from_str(include_str!(concat!(env!("OUT_DIR"), "/oxlintrc.json")))
    .expect("embedded JSON should be valid");

  if let Some(obj) = config.as_object_mut() {
    obj.remove("$schema");

    let has_next = has_package(packages, "next");
    let has_react = has_next || has_package(packages, "react");

    if let Some(plugins) = obj.get_mut("plugins").and_then(Value::as_array_mut) {
      if has_react {
        push_plugin(plugins, "react");
        push_plugin(plugins, "react-perf");
      }

      if has_next {
        push_plugin(plugins, "nextjs");
      }
    }
  }

  config
}

fn build_oxlint_config(packages: &Packages) -> String {
  let config =
    to_string_pretty(&build_oxlint_config_value(packages)).expect("serialization should succeed");
  format!("import {{ defineConfig }} from 'oxlint';\n\nexport default defineConfig({config});\n")
}

fn build_oxfmt_config() -> String {
  let config = strip_schema(include_str!(concat!(env!("OUT_DIR"), "/oxfmtrc.json")));
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

fn generate_ox_config(packages: Packages) -> Result<()> {
  write("oxlint.config.ts", build_oxlint_config(&packages))?;
  write("oxfmt.config.ts", build_oxfmt_config())?;

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
