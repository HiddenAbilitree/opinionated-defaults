use {
  anyhow::Result,
  jsonc_parser::parse_to_serde_value,
  serde::Deserialize,
  serde_json::{Map, Value, from_value},
  std::{ffi::OsStr, process::Command, slice::Iter},
};

#[derive(PartialEq)]
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
  pub fn iterator() -> Iter<'static, PackageManager> {
    static PACKAGE_MANAGERS: [PackageManager; 6] = [
      PackageManager::Bun,
      PackageManager::BunOld,
      PackageManager::Deno,
      PackageManager::Npm,
      PackageManager::Yarn,
      PackageManager::Pnpm,
    ];
    PACKAGE_MANAGERS.iter()
  }

  // unreachable is fine since this is only called when
  // we know that the lockfile name is one of the following
  pub fn from_lockfile(lockfile_path: &OsStr) -> PackageManager {
    match lockfile_path.to_str().unwrap() {
      "bun.lock" => PackageManager::Bun,
      "bun.lockb" => PackageManager::BunOld,
      "deno.lock" => PackageManager::Deno,
      "package-lock.json" => PackageManager::Npm,
      "yarn.lock" => PackageManager::Yarn,
      "pnpm-lock.yaml" => PackageManager::Pnpm,
      _ => unreachable!(),
    }
  }

  pub fn parse_lockfile(&self, contents: String) -> Result<JSONLockfile> {
    match *self {
      PackageManager::Bun => {
        // bun.lock is jsonc and not json so we cannot use serde_json's parser
        let data: JSONLockfile = parse_to_serde_value(&contents, &Default::default())?
          .and_then(|value| from_value(value).ok())
          .unwrap();
        Ok(data)
      }
      PackageManager::BunOld => {
        let data: PackageJSON = serde_json::from_str(&contents)?;
        let packages: Packages = data
          .dependencies
          .into_iter()
          .chain(data.dev_dependencies)
          .chain(data.peer_dependencies)
          .collect();

        Ok(JSONLockfile { packages })
      }
      PackageManager::Deno => todo!(),
      PackageManager::Npm => {
        let data: JSONLockfile = serde_json::from_str(&contents)?;
        Ok(data)
      }
      PackageManager::Yarn => todo!(),
      PackageManager::Pnpm => todo!(),
    }
  }

  pub fn lockfile(&self) -> &str {
    match *self {
      PackageManager::Bun => "bun.lock",
      PackageManager::BunOld => "bun.lockb",
      PackageManager::Deno => "deno.lock",
      PackageManager::Npm => "package-lock.json",
      PackageManager::Yarn => "yarn.lock",
      PackageManager::Pnpm => "pnpm-lock.yaml",
    }
  }

  pub fn cli(&self) -> &str {
    match *self {
      PackageManager::Bun => "bun",
      PackageManager::BunOld => "bun",
      PackageManager::Deno => "deno",
      PackageManager::Npm => "npm",
      PackageManager::Yarn => "yarn",
      PackageManager::Pnpm => "pnpm",
    }
  }
  pub fn command(&self) -> Command {
    let mut cmd = Command::new(self.cli());

    if *self == PackageManager::Deno {
      cmd
        .arg("add")
        .arg("npm:@hiddenability/opinionated-defaults@latest")
        .arg("npm:@types/node");
      return cmd;
    }

    let (i, dev) = match self {
      PackageManager::Npm => ("i", "-D"),
      PackageManager::Bun | PackageManager::BunOld => ("add", "-d"),
      _ => ("add", "-D"),
    };

    cmd
      .arg(i)
      .arg("@hiddenability/opinionated-defaults@latest")
      .arg("@types/node")
      .arg(dev);

    if *self == PackageManager::BunOld {
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
pub struct PackageJSON {
  pub dependencies: Packages,
  #[serde(rename = "devDependencies")]
  pub dev_dependencies: Packages,
  #[serde(rename = "peerDependencies")]
  pub peer_dependencies: Packages,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TSConfig {
  pub compiler_options: Option<CompilerOptions>,
}

#[derive(Deserialize)]
pub struct CompilerOptions {
  pub paths: Option<Map<String, Value>>,
}

pub struct Dependencies {
  pub packages: Map<String, Value>,
  pub valid_deps: Vec<(String, String)>,
  pub default_deps: Vec<String>,
}
