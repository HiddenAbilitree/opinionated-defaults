/*
 * TODO:
 * switch to wrapped-wrapped config object for eslint ({ prettier: true } or smth)
 */
use {
  anyhow::{Result, anyhow},
  ignore::Walk,
  jsonc_parser::parse_to_serde_value,
  serde::Deserialize,
  serde_json::{Map, Value, from_value},
  std::{
    collections::BTreeSet,
    env::current_dir,
    fs::{read_to_string, write},
    path::PathBuf,
  },
};

#[derive(Deserialize)]
struct RepoData {
  packages: Map<String, Value>,
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
  let eslint_imports = handle_dependencies(Muehehehe {
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

  let eslint_spread: Vec<String> = eslint_imports
    .iter()
    .map(|dep| format!("...{dep}"))
    .collect();

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
    eslint_spread.join(",\n  ")
  );

  write("eslint.config.ts", eslint_config)?;

  let prettier_imports = handle_dependencies(Muehehehe {
    packages,
    valid_deps: &[("tailwindcss", "prettierConfigTailwind")],
    default_deps: &["prettierConfigBase"],
  })?;

  let mut prettier_objects = prettier_imports.clone();

  if prettier_imports.contains(&String::from("prettierConfigTailwind")) {
    let tailwind_path = String::from(
      find_tailwind_file()
        .expect("couldnt find ur tailwind file")
        .to_str()
        .expect("uhh invalid tailwind file or smth idk"),
    );
    prettier_objects.push(format!(r"{{ tailwindStylesheet: `{tailwind_path}`, }}"));
  }

  let prettier_config = format!(
    r#"import {{
  prettierConfig,
  {},
}} from '@hiddenability/opinionated-defaults/prettier';

export default prettierConfig(
  {},
);
"#,
    prettier_imports.join(",\n  "),
    prettier_objects.join(",\n  "),
  );

  std::process::Command::new("bun")
    .arg("add")
    .arg("@hiddenability/opinionated-defaults@latest")
    .arg("-d")
    .output()
    .expect("failed to spawn process");

  write("prettier.config.mjs", prettier_config)?;
  Ok(())
}

// from https://github.com/lbwa/package-json-rs/blob/ac69f7bbcd6ce97698a6ebf1da8c1976239dc8ad/src/fs.rs#L10C1-L25C2
fn find_lockfile() -> Result<PathBuf> {
  let mut current_dir = PathBuf::from(current_dir().as_ref().expect("probably no permissions"));
  loop {
    let path = current_dir.join("bun.lock");
    if path.exists() {
      return Ok(path);
    }
    if !current_dir.pop() {
      return Err(anyhow!("no lockfile found"));
    }
  }
}

fn find_tailwind_file() -> Result<PathBuf> {
  let binding = current_dir().expect("probably no permissions");
  let base_dir = binding.as_path();
  for result in Walk::new(base_dir).filter_map(|e| e.ok()) {
    if !result.file_type().unwrap().is_file() {
      continue;
    }
    let path = result.path();

    if !path.exists() || path.extension().expect("probably no permissions") != "css" {
      continue;
    }

    let contents = read_to_string(path).expect("invalid tailwind path lowkey my fault");

    if contents.contains(r#"import "tailwindcss";"#) {
      return Ok(PathBuf::from(
        path.strip_prefix(base_dir).expect("couldnt remove prefix"),
      ));
    }
  }
  Err(anyhow!("couldnt find a tailwind file"))
}

fn main() -> Result<()> {
  let content = read_to_string(find_lockfile().expect("couldnt find ur lockfile"))
    .expect("couldnt read ur lockfile, prob my fault");
  // bun.lock is jsonc and not json so we cannot use serde_json's parser
  let data: RepoData = parse_to_serde_value(&content, &Default::default())?
    .and_then(|value| from_value(value).ok())
    .expect("couldnt parse ur lockfile");

  generate_config(&data.packages)?;

  Ok(())
}
