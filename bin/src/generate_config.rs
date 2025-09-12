use {
  crate::{
    handle_dependencies::handle_dependencies,
    types::{Dependencies, Gitignore, TSConfig},
    utils::{find_file, find_tailwind_file},
  },
  anyhow::Result,
  jsonc_parser::parse_to_serde_value,
  pathdiff::diff_paths,
  serde_json::{Map, Value, from_value},
  std::{
    env::current_dir,
    fs::{read_to_string, write},
  },
};

pub fn generate_config(packages: Map<String, Value>) -> Result<()> {
  let mut eslint_imports = handle_dependencies(Dependencies {
    packages: packages.clone(),
    valid_deps: vec![
      ("astro".to_string(), "eslintConfigAstro".to_string()),
      ("react".to_string(), "eslintConfigReact".to_string()),
      ("solid-js".to_string(), "eslintConfigSolid".to_string()),
      ("turborepo".to_string(), "eslintConfigTurbo".to_string()),
      ("next".to_string(), "eslintConfigNext".to_string()),
    ],
    default_deps: vec![
      "eslintConfigBase".to_string(),
      "eslintConfigPerfectionist".to_string(),
      "eslintConfigPrettier".to_string(),
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
        eprintln!("❌ Could not parse tsconfig.json. Skipping relative check...");
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
import {{ fileURLToPath }} from 'node:url';
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

  let prettier_imports = handle_dependencies(Dependencies {
    packages,
    valid_deps: vec![(
      "tailwindcss".to_string(),
      "prettierConfigTailwind".to_string(),
    )],
    default_deps: vec!["prettierConfigBase".to_string()],
  })?;

  let mut prettier_objects = prettier_imports.clone();

  if prettier_imports.contains(&"prettierConfigTailwind".to_string()) {
    if let Some(tailwind_path) = find_tailwind_file() {
      let tailwind_path = tailwind_path.to_str().unwrap().to_string();
      prettier_objects.push(format!(
        r"{{
  tailwindStylesheet: `./{tailwind_path}`,
}}"
      ));
    } else {
      eprintln!(
        "⚠️ TailwindCSS dependency found but could not find a relevant css file. Skipping..."
      );
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
