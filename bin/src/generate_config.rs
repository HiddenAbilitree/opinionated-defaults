use {
  crate::{
    handle_dependencies::handle_dependencies,
    monorepo::{
      build_oxfmt_config, build_oxlint_config, find_workspace_projects, remove_child_ox_configs,
    },
    types::{Dependencies, PackageManager, Packages, TSConfig, Tooling},
    utils::{find_file, find_files, find_tailwind_file},
  },
  anyhow::{Context, Result, bail},
  jsonc_parser::{ParseOptions, parse_to_serde_value},
  pathdiff::diff_paths,
  serde_json::{Map, Value, from_str, from_value, to_string_pretty},
  std::{
    env::current_dir,
    fmt::Write,
    fs::{read_to_string, remove_file, write},
    io::{ErrorKind, Write as IoWrite},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
  },
};

fn dep(pkg: &str, import: &str) -> (String, String) {
  (pkg.into(), import.into())
}

fn eslint_config_path(import: &str) -> Option<&'static str> {
  match import {
    "eslintConfigAstro" => Some("astro"),
    "eslintConfigBetterTailwindcss" => Some("better-tailwindcss"),
    "eslintConfigFunctional" => Some("functional"),
    "eslintConfigNext" => Some("next"),
    "eslintConfigOxlint" => Some("oxlint"),
    "eslintConfigPerfectionist" => Some("perfectionist"),
    "eslintConfigPrettier" => Some("prettier"),
    "eslintConfigReact" => Some("react"),
    "eslintConfigRelative" => Some("relative"),
    "eslintConfigSolid" => Some("solid"),
    "eslintConfigTurbo" => Some("turbo"),
    _ => None,
  }
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
  for import in imports
    .iter()
    .filter(|import| eslint_config_path(import).is_none())
  {
    writeln!(out, "  {import},").unwrap();
  }
  writeln!(out, "}} from '@hiddenability/opinionated-defaults/eslint';").unwrap();
  for import in imports {
    if let Some(path) = eslint_config_path(import) {
      writeln!(
        out,
        "import {import} from '@hiddenability/opinionated-defaults/eslint/{path}';"
      )
      .unwrap();
    }
  }
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
fn formatter_command(manager: PackageManager, tooling: Tooling) -> Command {
  let formatter = match tooling {
    Tooling::Eslint => "prettier",
    Tooling::Ox => "oxfmt",
  };
  let mut command = match manager {
    PackageManager::Bun | PackageManager::BunOld => {
      let mut command = Command::new("bun");
      command.args(["run", "--bun"]);
      command
    }
    PackageManager::Deno => {
      let mut command = Command::new("deno");
      command.args(["run", "-A"]);
      command
    }
    PackageManager::Npm => {
      let mut command = Command::new("npx");
      command.arg("--no-install");
      command
    }
    PackageManager::Pnpm => {
      let mut command = Command::new("pnpm");
      command.arg("exec");
      command
    }
    PackageManager::Yarn => {
      let mut command = Command::new("yarn");
      command.arg("exec");
      command
    }
  };

  if manager == PackageManager::Deno {
    command.arg(format!("npm:{formatter}"));
  } else {
    command.arg(formatter);
  }

  command
}

fn format_config_source(
  manager: PackageManager,
  tooling: Tooling,
  config_path: &Path,
  ignore_path: &Path,
  source_path: &Path,
  source: &str,
) -> Result<String> {
  let mut command = formatter_command(manager, tooling);
  command
    .arg("--config")
    .arg(config_path)
    .arg("--ignore-path")
    .arg(ignore_path)
    .arg("--stdin-filepath")
    .arg(source_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  let mut child = command
    .spawn()
    .with_context(|| format!("could not start formatter for {}", source_path.display()))?;
  child
    .stdin
    .take()
    .expect("formatter stdin should be piped")
    .write_all(source.as_bytes())
    .with_context(|| format!("could not send {} to formatter", source_path.display()))?;
  let output = child
    .wait_with_output()
    .with_context(|| format!("could not format {}", source_path.display()))?;

  if !output.status.success() {
    bail!(
      "could not format {}:\n{}",
      source_path.display(),
      String::from_utf8_lossy(&output.stderr).trim()
    );
  }

  String::from_utf8(output.stdout).with_context(|| {
    format!(
      "formatter returned invalid UTF-8 for {}",
      source_path.display()
    )
  })
}

fn format_generated_configs(
  manager: PackageManager,
  tooling: Tooling,
  formatter_path: &Path,
  formatter_config: &str,
  configs: &[(&Path, &str)],
) -> Result<Vec<String>> {
  let nonce = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_nanos();
  let extension = formatter_path
    .extension()
    .and_then(|extension| extension.to_str())
    .unwrap_or("js");
  let temporary_name = format!(".opinionated-defaults-{}-{nonce}", std::process::id());
  let temporary_path = formatter_path.with_file_name(format!("{temporary_name}.{extension}"));
  let temporary_ignore_path = formatter_path.with_file_name(format!("{temporary_name}.ignore"));
  write(&temporary_path, formatter_config)
    .with_context(|| format!("could not stage {}", formatter_path.display()))?;
  if let Err(error) = write(&temporary_ignore_path, "") {
    let _ = remove_file(&temporary_path);
    return Err(error)
      .with_context(|| format!("could not stage {}", temporary_ignore_path.display()));
  }

  let formatted = configs
    .iter()
    .map(|(path, source)| {
      format_config_source(
        manager,
        tooling,
        &temporary_path,
        &temporary_ignore_path,
        path,
        source,
      )
    })
    .collect::<Result<Vec<_>>>();
  let config_cleanup = remove_file(&temporary_path)
    .with_context(|| format!("could not remove {}", temporary_path.display()));
  let ignore_cleanup = remove_file(&temporary_ignore_path)
    .with_context(|| format!("could not remove {}", temporary_ignore_path.display()));

  match (formatted, config_cleanup, ignore_cleanup) {
    (Err(error), _, _) | (Ok(_), Err(error), _) | (Ok(_), Ok(()), Err(error)) => Err(error),
    (Ok(formatted), Ok(()), Ok(())) => Ok(formatted),
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

fn generate_eslint_config(packages: Packages, manager: PackageManager) -> Result<()> {
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

  let eslint_path = cwd.join("eslint.config.ts");
  let prettier_path = cwd.join("prettier.config.mjs");
  let formatted = format_generated_configs(
    manager,
    Tooling::Eslint,
    &prettier_path,
    &prettier_config,
    &[
      (&eslint_path, &eslint_config),
      (&prettier_path, &prettier_config),
    ],
  )?;
  let mut formatted = formatted.into_iter();
  write(
    eslint_path,
    formatted.next().expect("eslint config should be formatted"),
  )?;
  write(
    prettier_path,
    formatted
      .next()
      .expect("prettier config should be formatted"),
  )?;
  ensure_ignore_pattern(".gitignore", GENERATED_TS_IGNORE_PATTERN)?;

  update_scripts(&[("lint", "eslint ."), ("lint:fix", "eslint . --fix")])?;

  Ok(())
}

fn generate_ox_config(packages: &Packages, manager: PackageManager) -> Result<()> {
  let root = current_dir()?;
  let projects = find_workspace_projects(&root)?;

  let oxlint_path = root.join("oxlint.config.ts");
  let oxfmt_path = root.join("oxfmt.config.ts");
  let oxlint_config = build_oxlint_config(packages, &projects);
  let oxfmt_config = build_oxfmt_config(packages, &projects);
  let formatted = format_generated_configs(
    manager,
    Tooling::Ox,
    &oxfmt_path,
    &oxfmt_config,
    &[(&oxlint_path, &oxlint_config), (&oxfmt_path, &oxfmt_config)],
  )?;
  let mut formatted = formatted.into_iter();
  write(
    oxlint_path,
    formatted.next().expect("oxlint config should be formatted"),
  )?;
  write(
    oxfmt_path,
    formatted.next().expect("oxfmt config should be formatted"),
  )?;
  remove_child_ox_configs(&root, &projects)?;

  update_scripts(&[
    ("lint", "oxlint"),
    ("lint:fix", "oxlint --fix"),
    ("format", "oxfmt ."),
    ("format:check", "oxfmt --check ."),
  ])?;

  Ok(())
}

pub fn generate_config(
  packages: Packages,
  tooling: Tooling,
  manager: PackageManager,
) -> Result<()> {
  match tooling {
    Tooling::Eslint => generate_eslint_config(packages, manager),
    Tooling::Ox => generate_ox_config(&packages, manager),
  }
}

#[cfg(test)]
mod tests {
  use {
    super::{
      DefaultProjectConfig, build_eslint_config, build_oxfmt_config, build_prettier_config,
      format_generated_configs,
    },
    crate::types::{PackageManager, Packages, Tooling},
    std::path::Path,
  };

  #[test]
  fn isolates_optional_eslint_config_imports() {
    let config = build_eslint_config(
      &[
        "eslintConfigAstro".into(),
        "eslintConfigBase".into(),
        "eslintConfigPrettier".into(),
      ],
      &[],
      DefaultProjectConfig {
        files: &[],
        default_project: None,
      },
    );

    assert!(config.contains(
      "import eslintConfigAstro from '@hiddenability/opinionated-defaults/eslint/astro';"
    ));
    assert!(config.contains(
      "import eslintConfigPrettier from '@hiddenability/opinionated-defaults/eslint/prettier';"
    ));
    assert!(!config.contains("  eslintConfigAstro,"));
    assert!(!config.contains("  eslintConfigPrettier,"));
  }
  #[test]
  fn formats_generated_files_with_generated_prettier_config() {
    let prettier_config = build_prettier_config(&["prettierConfigBase".into()], None);
    let formatted = format_generated_configs(
      PackageManager::Bun,
      Tooling::Eslint,
      Path::new("prettier.config.mjs"),
      &prettier_config,
      &[(
        Path::new("eslint.config.ts"),
        "const generated={value:\"config\"};\n",
      )],
    )
    .unwrap();

    assert_eq!(formatted, ["const generated = { value: 'config' };\n"]);
  }
  #[test]
  fn formats_generated_files_with_generated_oxfmt_config() {
    let oxfmt_config = build_oxfmt_config(&Packages::new(), &[]);
    let formatted = format_generated_configs(
      PackageManager::Bun,
      Tooling::Ox,
      Path::new("oxfmt.config.ts"),
      &oxfmt_config,
      &[(
        Path::new("oxlint.config.ts"),
        "const generated={value:\"config\"};\n",
      )],
    )
    .unwrap();

    assert_eq!(formatted, ["const generated = { value: 'config' };\n"]);
  }
}
