use {
  anyhow::Result,
  jsonc_parser::{ParseOptions, parse_to_serde_value},
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

include!(concat!(env!("OUT_DIR"), "/dependencies.rs"));

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

  pub fn parse_lockfile(self, contents: &str) -> Result<JSONLockfile> {
    match self {
      Self::Bun => {
        // bun.lock is jsonc and not json so we cannot use serde_json's parser
        parse_to_serde_value(contents, &ParseOptions::default())?
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

  pub fn command(self, solid: bool) -> Command {
    let mut cmd = Command::new(self.cli());

    if self == Self::Deno {
      cmd
        .arg("add")
        .arg("npm:@hiddenability/opinionated-defaults@latest");
      for dependency in OX_DEPENDENCIES {
        cmd.arg(format!("npm:{dependency}"));
      }
      cmd.arg("npm:oxlint-tsgolint").arg("npm:@types/node");
      if solid {
        cmd.arg(format!("npm:{SOLID_PLUGIN_DEPENDENCY}"));
      }
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
      .args(OX_DEPENDENCIES)
      .arg("oxlint-tsgolint");
    if solid {
      cmd.arg(SOLID_PLUGIN_DEPENDENCY);
    }
    cmd.arg("@types/node").arg(dev_flag);

    if self == Self::BunOld {
      cmd.arg("--save-text-lockfile");
      eprintln!("⚠️ Detected deprecated bun.lockb file. Make sure to delete it later.");
    }

    cmd
  }
}

/// `serde_json` map type `pub struct Map<K, V>`
/// represents a JSON key/value type
pub type Packages = Map<String, Value>;

pub fn has_solid(packages: &Packages) -> bool {
  packages.contains_key("solid-js")
    || packages.contains_key("@solidjs/start")
    || packages.contains_key("@tanstack/solid-start")
}

#[derive(Deserialize)]
pub struct JSONLockfile {
  pub packages: Packages,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Workspaces {
  Patterns(Vec<String>),
  Config { packages: Vec<String> },
}

impl Workspaces {
  pub fn patterns(&self) -> &[String] {
    match self {
      Self::Patterns(patterns) | Self::Config { packages: patterns } => patterns,
    }
  }
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
  pub workspaces: Option<Workspaces>,
}

impl PackageJSON {
  pub fn into_packages(self) -> Packages {
    self
      .dependencies
      .into_iter()
      .chain(self.dev_dependencies)
      .chain(self.peer_dependencies)
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn args(manager: PackageManager) -> Vec<String> {
    manager
      .command(false)
      .get_args()
      .map(|arg| arg.to_string_lossy().into_owned())
      .collect()
  }

  #[test]
  fn oxlint_installs_only_the_requested_solid_plugin() {
    for manager in PackageManager::ALL {
      let plugin = if manager == PackageManager::Deno {
        format!("npm:{SOLID_PLUGIN_DEPENDENCY}")
      } else {
        SOLID_PLUGIN_DEPENDENCY.to_string()
      };
      let with_solid: Vec<_> = manager
        .command(true)
        .get_args()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
      let without_solid = args(manager);

      assert!(!without_solid.iter().any(|arg| arg == &plugin));
      assert_eq!(with_solid.iter().filter(|arg| *arg == &plugin).count(), 1);
      assert_eq!(
        with_solid
          .into_iter()
          .filter(|arg| arg != &plugin)
          .collect::<Vec<_>>(),
        without_solid
      );
    }
  }
}
