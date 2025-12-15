use {
  anyhow::Result,
  jsonc_parser::parse_to_serde_value,
  serde::Deserialize,
  serde_json::{Map, Value, from_value},
  std::{ffi::OsStr, process::Command},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
  Bun,
  BunOld,
  Deno,
  Npm,
  Pnpm,
  Yarn,
}

pub struct ProjectData {
  pub packages: Packages,
  pub manager: PackageManager,
}

impl PackageManager {
  pub const ALL: [Self; 6] = [
    Self::Bun,
    Self::BunOld,
    Self::Deno,
    Self::Npm,
    Self::Yarn,
    Self::Pnpm,
  ];

  pub fn from_lockfile(lockfile_path: &OsStr) -> Self {
    match lockfile_path
      .to_str()
      .expect("lockfile path should be valid UTF-8")
    {
      "bun.lock" => Self::Bun,
      "bun.lockb" => Self::BunOld,
      "deno.lock" => Self::Deno,
      "package-lock.json" => Self::Npm,
      "yarn.lock" => Self::Yarn,
      "pnpm-lock.yaml" => Self::Pnpm,
      other => panic!("unrecognized lockfile: {other}"),
    }
  }

  pub fn parse_lockfile(&self, contents: &str) -> Result<JSONLockfile> {
    match self {
      Self::Bun => {
        // bun.lock is jsonc and not json so we cannot use serde_json's parser
        parse_to_serde_value(contents, &Default::default())?
          .and_then(|value| from_value(value).ok())
          .ok_or_else(|| anyhow::anyhow!("failed to parse bun.lock"))
      }
      Self::BunOld => {
        let data: PackageJSON = serde_json::from_str(contents)?;
        let packages: Packages = data
          .dependencies
          .into_iter()
          .chain(data.dev_dependencies)
          .chain(data.peer_dependencies)
          .collect();

        Ok(JSONLockfile { packages })
      }
      Self::Deno => todo!("deno lockfile parsing not implemented"),
      Self::Npm => Ok(serde_json::from_str(contents)?),
      Self::Yarn => todo!("yarn lockfile parsing not implemented"),
      Self::Pnpm => todo!("pnpm lockfile parsing not implemented"),
    }
  }

  pub const fn lockfile(self) -> &'static str {
    match self {
      Self::Bun => "bun.lock",
      Self::BunOld => "bun.lockb",
      Self::Deno => "deno.lock",
      Self::Npm => "package-lock.json",
      Self::Yarn => "yarn.lock",
      Self::Pnpm => "pnpm-lock.yaml",
    }
  }

  pub const fn cli(self) -> &'static str {
    match self {
      Self::Bun | Self::BunOld => "bun",
      Self::Deno => "deno",
      Self::Npm => "npm",
      Self::Yarn => "yarn",
      Self::Pnpm => "pnpm",
    }
  }

  pub fn command(self) -> Command {
    let mut cmd = Command::new(self.cli());

    if self == Self::Deno {
      cmd
        .arg("add")
        .arg("npm:@hiddenability/opinionated-defaults@latest")
        .arg("npm:@types/node");
      return cmd;
    }

    let (subcmd, dev_flag) = match self {
      Self::Npm => ("i", "-D"),
      Self::Bun | Self::BunOld => ("add", "-d"),
      _ => ("add", "-D"),
    };

    cmd
      .arg(subcmd)
      .arg("@hiddenability/opinionated-defaults@latest")
      .arg("@types/node")
      .arg(dev_flag);

    if self == Self::BunOld {
      cmd.arg("--save-text-lockfile");
      eprintln!("⚠️ Detected deprecated bun.lockb file. Make sure to delete it later.");
    }

    cmd
  }
}

/// serde_json map type `pub struct Map<K, V>`
/// represents a JSON key/value type
pub type Packages = Map<String, Value>;

#[derive(Deserialize)]
pub struct JSONLockfile {
  pub packages: Packages,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageJSON {
  #[serde(default)]
  pub dependencies: Packages,
  #[serde(default)]
  pub dev_dependencies: Packages,
  #[serde(default)]
  pub peer_dependencies: Packages,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TSConfig {
  pub compiler_options: Option<CompilerOptions>,
  pub include: Option<Vec<String>>,
  pub files: Option<Vec<String>>,
  pub references: Option<Value>,
}

impl TSConfig {
  pub fn is_solution_style(&self) -> bool {
    self.references.is_some()
      && (self.compiler_options.is_none() || self.files.as_ref().is_some_and(Vec::is_empty))
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
  pub paths: Option<Map<String, Value>>,
  #[serde(default)]
  pub allow_js: bool,
}

pub struct Dependencies {
  pub packages: Map<String, Value>,
  pub valid_deps: Vec<(String, String)>,
  pub default_deps: Vec<String>,
}
