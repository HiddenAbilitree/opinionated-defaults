use {
  crate::{
    monorepo::{
      build_oxfmt_config, build_oxlint_config, find_workspace_projects, remove_child_ox_configs,
    },
    types::{PackageManager, Packages},
  },
  anyhow::{Context, Result, bail},
  serde_json::{Map, Value, from_str, to_string_pretty},
  std::{
    env::current_dir,
    fs::{read_to_string, remove_file, write},
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
  },
};

fn formatter_command(manager: PackageManager) -> Command {
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
    command.arg("npm:oxfmt");
  } else {
    command.arg("oxfmt");
  }

  command
}

fn format_config_source(
  manager: PackageManager,
  config_path: &Path,
  ignore_path: &Path,
  source_path: &Path,
  source: &str,
) -> Result<String> {
  let mut command = formatter_command(manager);
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

pub fn generate_config(packages: &Packages, manager: PackageManager) -> Result<()> {
  let root = current_dir()?;
  let projects = find_workspace_projects(&root)?;

  let oxlint_path = root.join("oxlint.config.ts");
  let oxfmt_path = root.join("oxfmt.config.ts");
  let oxlint_config = build_oxlint_config(packages, &projects);
  let oxfmt_config = build_oxfmt_config(packages, &projects);
  let formatted = format_generated_configs(
    manager,
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

#[cfg(test)]
mod tests {
  use {
    super::{build_oxfmt_config, format_generated_configs},
    crate::types::{PackageManager, Packages},
    std::path::Path,
  };

  #[test]
  fn formats_generated_files_with_generated_oxfmt_config() {
    let oxfmt_config = build_oxfmt_config(&Packages::new(), &[]);
    let formatted = format_generated_configs(
      PackageManager::Bun,
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
