use jsonc_parser::parse_to_serde_value;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepoData {
  packages: Map<String, Value>,
}

// from https://stackoverflow.com/a/38406885/23438710
fn first_uppercase(s: &str) -> String {
  let mut c = s.chars();
  match c.next() {
    None => String::new(),
    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
  }
}

struct Muehehehe<'a> {
  packages: &'a Map<String, Value>,
  valid_deps: &'a [&'a str],
  default_deps: &'a [&'a str],
  variant: &'a str,
}

fn handle_dependencies(
  Muehehehe {
    packages,
    valid_deps,
    default_deps,
    variant,
  }: Muehehehe,
) -> Result<(Vec<String>, Vec<String>), Box<dyn Error>> {
  let mut present_deps: BTreeSet<String> = default_deps.iter().map(|dep| dep.to_string()).collect();

  for dep in valid_deps {
    if packages.contains_key(*dep) {
      present_deps.insert(dep.to_string());
    }
  }

  let imports: Vec<String> = present_deps
    .iter()
    .map(|dep| format!("{variant}Config{}", first_uppercase(dep)))
    .collect();

  let object_spread: Vec<String> = present_deps
    .iter()
    .map(|dep| format!("...{variant}Config{}", first_uppercase(dep)))
    .collect();

  Ok((imports, object_spread))
}

fn generate_config(packages: &Map<String, Value>) -> Result<(), Box<dyn Error>> {
  let (eslint_imports, eslint_spread) = handle_dependencies(Muehehehe {
    packages,
    valid_deps: &["astro", "elysia", "react", "solid-js", "turborepo", "next"],
    default_deps: &["base", "perfectionist", "prettier"],
    variant: "eslint",
  })?;

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

  std::fs::write("eslint.config.ts", eslint_config)?;

  let (prettier_imports, _) = handle_dependencies(Muehehehe {
    packages,
    valid_deps: &["tailwindcss"],
    default_deps: &["base"],
    variant: "prettier",
  })?;

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
    prettier_imports.join(",\n  "),
  );

  std::fs::write("prettier.config.mjs", prettier_config)?;
  Ok(())
}

// from https://github.com/lbwa/package-json-rs/blob/ac69f7bbcd6ce97698a6ebf1da8c1976239dc8ad/src/fs.rs#L10C1-L25C2
pub fn find_lockfile() -> Result<PathBuf, Box<dyn Error>> {
  let mut current_dir = PathBuf::from(std::env::current_dir().as_ref().unwrap());
  loop {
    let path = current_dir.join("bun.lock");
    if path.exists() {
      return Ok(path);
    }
    if !current_dir.pop() {
      panic!("erm... no bun.lock...");
    }
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  let content = std::fs::read_to_string(find_lockfile().unwrap()).unwrap();
  // bun.lock is jsonc and not json so we cannot use serde_json's parser
  let data: RepoData = parse_to_serde_value(&content, &Default::default())
    .map_err(|_| "ermm bun.lock is cooked!")?
    .and_then(|value| serde_json::from_value(value).ok())
    .ok_or("ermm.... malformed bun.lock or smth")?;

  generate_config(&data.packages)?;

  Ok(())
}
