/*
 * TODO:
 * switch to wrapped-wrapped config object for eslint ({ prettier: true } or smth)
 */
use {
  crate::utils::{find_file, find_tailwind_file},
  anyhow::Result,
  jsonc_parser::parse_to_serde_value,
  pathdiff::diff_paths,
  serde::Deserialize,
  serde_json::{Map, Value, from_value},
  std::{
    collections::BTreeSet,
    env::current_dir,
    fs::{read_to_string, write},
    process::Command,
    time::Instant,
  },
};

mod utils;

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

struct Gitignore {
  import: String,
  var: String,
  config: String,
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

  if let Some(tsconfig) = find_file("tsconfig.json") {
    let tsconfig_contents = read_to_string(tsconfig)?;
    let parsed_config = parse_to_serde_value(&tsconfig_contents, &Default::default())?
      .and_then(|value| from_value::<TSConfig>(value).ok());

    match parsed_config {
      Some(config) => {
        if config
          .compiler_options
          .and_then(|opts| opts.paths)
          .is_some_and(|paths| !paths.is_empty())
        {
          eslint_imports.push("eslintConfigRelative".to_string());
        }
      }
      None => {
        eprintln!("Could not parse tsconfig.json. Skipping relative check...");
      }
    }
  }

  let gitignore: Gitignore = match find_file(".gitignore") {
    Some(path) => {
      let var_string = format!(
        "\nconst gitignorePath = fileURLToPath(new URL(`{}`, import.meta.url));\n",
        diff_paths(&path, current_dir()?).unwrap().to_str().unwrap()
      );
      Gitignore {
        import: "import { includeIgnoreFile } from '@eslint/compat';\n".to_string(),
        var: var_string,
        config: "includeIgnoreFile(gitignorePath, `Imported .gitignore patterns`),\n  ".to_string(),
      }
    }
    None => Gitignore {
      import: "".to_string(),
      var: "".to_string(),
      config: "".to_string(),
    },
  };

  let eslint_config = format!(
    r#"{}import {{
  eslintConfig,
  {},
}} from '@hiddenability/opinionated-defaults/eslint';
import {{ fileURLToPath, URL }} from 'node:url';
{}
export default eslintConfig([
  {}{},
]);
"#,
    gitignore.import,
    eslint_imports.join(",\n  "),
    gitignore.var,
    gitignore.config,
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

  if prettier_imports.contains(&"prettierConfigTailwind".to_string()) {
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

fn main() -> Result<()> {
  let before = Instant::now();

  env_logger::init();

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
    Some(something) => something,
    None => {
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
