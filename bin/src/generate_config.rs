use {
  crate::{
    handle_dependencies::handle_dependencies,
    types::{Dependencies, Packages, TSConfig},
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

fn build_eslint_config(
  imports: &[String],
  gitignore_paths: &[&Path],
  default_project_files: &[&str],
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
      let files_str = default_project_files
        .iter()
        .map(|f| format!("`{f}`"))
        .collect::<Vec<_>>()
        .join(", ");
      writeln!(out, "  ...{import}([{files_str}]),").unwrap();
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
      ", {{\n  tailwindStylesheet: `./{}`\n}}",
      path.display()
    )
    .unwrap();
  }
  writeln!(out, ");").unwrap();

  out
}

pub fn generate_config(packages: Packages) -> Result<()> {
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

  let eslint_config = build_eslint_config(&eslint_imports, &gitignore_refs, &default_project_files);

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

  update_scripts()?;

  Ok(())
}

fn update_scripts() -> Result<()> {
  let path = "package.json";
  if let Ok(contents) = read_to_string(path) {
    let mut v: Value = from_str(&contents)?;

    if let Some(scripts) = v.get_mut("scripts") {
      if let Some(map) = scripts.as_object_mut() {
        map
          .entry("lint")
          .or_insert(Value::String("eslint .".into()));
        map
          .entry("lint:fix")
          .or_insert(Value::String("eslint . --fix".into()));
      }
    } else {
      let mut map = Map::new();
      map.insert("lint".into(), Value::String("eslint .".into()));
      map.insert("lint:fix".into(), Value::String("eslint . --fix".into()));

      if let Some(obj) = v.as_object_mut() {
        obj.insert("scripts".into(), Value::Object(map));
      }
    }

    let new_contents = to_string_pretty(&v)? + "\n";
    write(path, new_contents)?;
  }
  Ok(())
}
