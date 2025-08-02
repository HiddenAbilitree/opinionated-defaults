/*
 * TODO:
 * switch to wrapped-wrapped config object for eslint ({ prettier: true } or smth)
 */
use {
  anyhow::{Result, anyhow},
  ignore::Walk,
  jsonc_parser::parse_to_serde_value,
  regex::RegexSet,
  serde::Deserialize,
  serde_json::{Map, Value, from_value},
  std::{
    collections::BTreeSet,
    env::current_dir,
    fs::{read_to_string, write},
    path::PathBuf,
    process::Command,
    time::Instant,
  },
};

#[derive(Deserialize)]
struct RepoData {
  packages: Map<String, Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TSConfig {
  compiler_options: Option<CompilerOptions>,
}

#[derive(Deserialize)]
struct CompilerOptions {
  paths: Option<Map<String, Value>>,
}

struct Muehehehe<'a> {
  packages: &'a Map<String, Value>,
  valid_deps: &'a [(&'a str, &'a str)],
  default_deps: &'a [&'a str],
}

fn handle_dependencies(
  Muehehehe {
    packages,
    valid_deps,
    default_deps,
  }: Muehehehe,
) -> Result<Vec<String>> {
  let mut present_deps: BTreeSet<String> = default_deps.iter().map(|dep| dep.to_string()).collect();

  for dep in valid_deps {
    let (k, v) = dep;
    if packages.contains_key(*k) {
      present_deps.insert(v.to_string());
    }
  }

  let imports: Vec<String> = present_deps.iter().cloned().collect();

  Ok(imports)
}

fn generate_config(packages: &Map<String, Value>) -> Result<()> {
  let mut eslint_imports = handle_dependencies(Muehehehe {
    packages,
    valid_deps: &[
      ("astro", "eslintConfigAstro"),
      ("elysia", "eslintConfigElysia"),
      ("react", "eslintConfigReact"),
      ("solid-js", "eslintConfigSolid"),
      ("turborepo", "eslintConfigTurbo"),
      ("next", "eslintConfigNext"),
    ],
    default_deps: &[
      "eslintConfigBase",
      "eslintConfigPerfectionist",
      "eslintConfigPrettier",
    ],
  })?;

  let tsconfig = read_to_string(find_file("tsconfig.json").unwrap()).unwrap();

  let tsconfig_data: TSConfig = match parse_to_serde_value(&tsconfig, &Default::default())?
    .and_then(|value| from_value(value).ok())
  {
    Some(data) => data,
    None => {
      eprintln!("Could not parse tsconfig.json");
      return Ok(());
    }
  };

  if let Some(compiler_options) = tsconfig_data.compiler_options
    && let Some(paths) = compiler_options.paths
    && !paths.is_empty()
  {
    eslint_imports.push("eslintConfigRelative".to_string());
  }

  let eslint_config = format!(
    r#"import {{ includeIgnoreFile }} from '@eslint/compat';
import {{
  eslintConfig,
  {},
}} from '@hiddenability/opinionated-defaults/eslint';
import {{ fileURLToPath, URL }} from 'node:url';

const gitignorePath = fileURLToPath(new URL(`.gitignore`, import.meta.url));

export default eslintConfig([
  includeIgnoreFile(gitignorePath, `Imported .gitignore patterns`),
  {},
]);
"#,
    eslint_imports.join(",\n  "),
    eslint_imports
      .iter()
      .map(|dep| format!("...{dep}"))
      .collect::<Vec<String>>()
      .join(",\n  ")
  );

  let prettier_imports = handle_dependencies(Muehehehe {
    packages,
    valid_deps: &[("tailwindcss", "prettierConfigTailwind")],
    default_deps: &["prettierConfigBase"],
  })?;

  let mut prettier_objects = prettier_imports.clone();

  if prettier_imports.contains(&String::from("prettierConfigTailwind")) {
    if let Ok(tailwind_path) = find_tailwind_file() {
      let tailwind_path = tailwind_path.to_str().unwrap().to_string();
      prettier_objects.push(format!(
        r"{{
  tailwindStylesheet: `./{tailwind_path}`,
}}"
      ));
    } else {
      eprintln!("TailwindCSS dependency found but could not find a relevant css file. Skipping...")
    }
  }

  let prettier_config = format!(
    r#"import {{
  prettierConfig,
  {},
}} from '@hiddenability/opinionated-defaults/prettier';

export default prettierConfig({});
"#,
    prettier_imports.join(",\n  "),
    prettier_objects.join(", "),
  );

  write("eslint.config.ts", eslint_config)?;
  write("prettier.config.mjs", prettier_config)?;
  Ok(())
}

// from https://github.com/lbwa/package-json-rs/blob/ac69f7bbcd6ce97698a6ebf1da8c1976239dc8ad/src/fs.rs#L10C1-L25C2
fn find_file(filename: &str) -> Result<PathBuf> {
  let mut current_dir = PathBuf::from(current_dir().as_ref().expect("probably no permissions"));
  loop {
    let path = current_dir.join(filename);
    if path.exists() {
      return Ok(path);
    }
    if !current_dir.pop() {
      return Err(anyhow!("no lockfile found"));
    }
  }
}

fn find_tailwind_file() -> Result<PathBuf> {
  let binding = current_dir()?;
  let base_dir = binding.as_path();

  let tailwind_regex = RegexSet::new([r#"@import ["']tailwindcss["'];"#, r#"@tailwind base;"#])?;

  for result in Walk::new(base_dir).filter_map(|e| e.ok()) {
    if let Some(file_type) = result.file_type()
      && !file_type.is_file()
    {
      continue;
    }

    let path = result.path();

    if !path.exists() {
      continue;
    }

    if let Some(extension) = path.extension()
      && extension != "css"
    {
      continue;
    }

    let contents = match read_to_string(path) {
      Ok(output) => output,
      Err(_) => continue,
    };

    if tailwind_regex.is_match(&contents) {
      return Ok(PathBuf::from(path.strip_prefix(base_dir)?));
    }
  }
  Err(anyhow!("couldnt find a tailwind file"))
}

fn main() -> Result<()> {
  let before = Instant::now();

  if Command::new("bun")
    .arg("add")
    .arg("@hiddenability/opinionated-defaults@latest")
    .arg("@types/node")
    .arg("-d")
    .arg("--save-text-lockfile")
    .output()
    .is_err()
  {
    eprintln!("Could not install @hiddenability/opinionated-defaults@latest with bun.");
    return Ok(());
  }

  let content = read_to_string(match find_file("bun.lock") {
    Ok(something) => something,
    Err(_) => {
      eprintln!("Could not find a valid lockfile.");
      return Ok(());
    }
  });

  let content = match content {
    Ok(content) => content,
    Err(_) => {
      eprintln!("Could not read the lockfile.");
      return Ok(());
    }
  };

  // bun.lock is jsonc and not json so we cannot use serde_json's parser
  let data: RepoData = match parse_to_serde_value(&content, &Default::default())?
    .and_then(|value| from_value(value).ok())
  {
    Some(data) => data,
    None => {
      eprintln!("Could not parse the lockfile.");
      return Ok(());
    }
  };

  generate_config(&data.packages)?;

  println!("✅ Done in {:.2?}", before.elapsed());

  Ok(())
}
