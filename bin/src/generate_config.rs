use {
  crate::{
    handle_dependencies::handle_dependencies,
    types::{Dependencies, Packages, TSConfig, Tooling},
    utils::{find_file, find_files, find_tailwind_file},
  },
  anyhow::Result,
  jsonc_parser::parse_to_serde_value,
  pathdiff::diff_paths,
  serde_json::{Map, Value, from_str, from_value, to_string_pretty},
  std::{
    env::current_dir,
    fmt::Write,
    fs::{read_to_string, write},
    path::Path,
  },
};

fn dep(pkg: &str, import: &str) -> (String, String) {
  (pkg.into(), import.into())
}

struct DefaultProjectConfig<'a> {
  files: &'a [&'a str],
  default_project: Option<&'a str>,
}

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

fn strip_schema(json: &str) -> String {
  let mut config: Value = from_str(json).expect("embedded JSON should be valid");
  if let Some(obj) = config.as_object_mut() {
    obj.remove("$schema");
  }
  to_string_pretty(&config).expect("serialization should succeed")
}

fn build_oxlint_config() -> String {
  let config = strip_schema(include_str!(concat!(env!("OUT_DIR"), "/oxlintrc.json")));
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

  let mut default_project_files: Vec<&str> = Vec::new();
  let mut default_project_tsconfig: Option<&str> = None;

  if let Some(tsconfig_path) = find_file("tsconfig.json") {
    let contents = read_to_string(tsconfig_path)?;
    let parsed = parse_to_serde_value(&contents, &Default::default())?
      .and_then(|value| from_value::<TSConfig>(value).ok());

    if let Some(config) = parsed {
      let has_paths = config
        .compiler_options
        .as_ref()
        .and_then(|opts| opts.paths.as_ref())
        .is_some_and(|paths| !paths.is_empty());

      if has_paths {
        eslint_imports.push("eslintConfigRelative".into());
      }

      default_project_tsconfig = find_default_project_tsconfig(&config);

      let has_empty_files = config.files.as_ref().is_some_and(|f| f.is_empty());
      let allow_js = config
        .compiler_options
        .as_ref()
        .is_some_and(|opts| opts.allow_js);

      let includes_ts = config.include.as_ref().is_none_or(|includes| {
        includes.iter().any(|pattern| {
          pattern == "."
            || pattern == "*"
            || pattern == "*.ts"
            || pattern.contains("**/*.ts")
            || pattern == "eslint.config.ts"
        })
      });

      let includes_mjs = (allow_js && config.include.is_none())
        || config.include.as_ref().is_some_and(|includes| {
          includes.iter().any(|pattern| {
            pattern == "."
              || pattern == "*"
              || pattern == "*.mjs"
              || pattern.contains("**/*.mjs")
              || pattern == "prettier.config.mjs"
          })
        });

      if has_empty_files || !includes_ts {
        default_project_files.push("eslint.config.ts");
      }

      if has_empty_files || !includes_mjs {
        default_project_files.push("prettier.config.mjs");
      }

      if !includes_mjs && find_file("postcss.config.mjs").is_some() {
        default_project_files.push("postcss.config.mjs");
      }

      if !default_project_files.is_empty() {
        eslint_imports.push("eslintConfigDefaultProject".into());
      }
    } else {
      eprintln!("❌ Could not parse tsconfig.json. Skipping relative check...");
    }
  }

  eslint_imports.sort();

  let cwd = current_dir()?;
  let gitignore_paths: Vec<_> = find_files(".gitignore")
    .into_iter()
    .filter_map(|path| diff_paths(&path, &cwd))
    .collect();
  let gitignore_refs: Vec<_> = gitignore_paths.iter().map(|p| p.as_path()).collect();

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

  let tailwind_path = if prettier_imports
    .iter()
    .any(|s| s == "prettierConfigTailwind")
  {
    match find_tailwind_file() {
      Some(path) => Some(path),
      None => {
        eprintln!(
          "⚠️ TailwindCSS dependency found but could not find a relevant css file. Skipping..."
        );
        None
      }
    }
  } else {
    None
  };

  let prettier_config = build_prettier_config(&prettier_imports, tailwind_path.as_deref());

  write("eslint.config.ts", eslint_config)?;
  write("prettier.config.mjs", prettier_config)?;

  update_scripts(&[
    ("lint", "eslint ."),
    ("lint:fix", "eslint . --fix"),
  ])?;

  Ok(())
}

fn generate_ox_config() -> Result<()> {
  write("oxlint.config.ts", build_oxlint_config())?;
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
    Tooling::Ox => generate_ox_config(),
  }
}

fn update_scripts(scripts: &[(&str, &str)]) -> Result<()> {
  let path = "package.json";
  if let Ok(contents) = read_to_string(path) {
    let mut v: Value = from_str(&contents)?;

    if let Some(map) = v
      .get_mut("scripts")
      .and_then(|s| s.as_object_mut())
    {
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
