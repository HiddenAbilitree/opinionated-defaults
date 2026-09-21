use std::{
  env::var,
  fmt::Write as _,
  fs::{read_to_string, write},
  path::PathBuf,
};

use serde_json::Value;

fn main() {
  let out_dir = var("OUT_DIR").unwrap();
  let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();

  generate_dependencies(&manifest_dir, &out_dir);
}

fn generate_dependencies(manifest_dir: &str, out_dir: &str) {
  let package_paths = [
    PathBuf::from(manifest_dir).join("../package.json"),
    PathBuf::from(manifest_dir).join("src/package.json"),
  ];
  let package_path = package_paths
    .iter()
    .find(|path| path.is_file())
    .unwrap_or_else(|| {
      panic!(
        "could not find package.json at {} or {}",
        package_paths[0].display(),
        package_paths[1].display()
      )
    });
  let package: Value = serde_json::from_str(
    &read_to_string(package_path)
      .unwrap_or_else(|error| panic!("could not read {}: {error}", package_path.display())),
  )
  .unwrap_or_else(|error| panic!("could not parse {}: {error}", package_path.display()));

  let dependencies = package
    .get("peerDependencies")
    .and_then(Value::as_object)
    .unwrap_or_else(|| panic!("peerDependencies missing from {}", package_path.display()));

  let dependency = |name: &str| {
    let version = dependencies
      .get(name)
      .and_then(Value::as_str)
      .unwrap_or_else(|| panic!("peer dependency {name} must have a string version"));
    serde_json::to_string(&format!("{name}@{version}"))
      .unwrap_or_else(|error| panic!("could not encode dependency {name}: {error}"))
  };

  let mut generated = String::from("const OX_DEPENDENCIES: &[&str] = &[\n");
  for name in ["oxlint", "oxfmt"] {
    writeln!(generated, "  {},", dependency(name)).unwrap();
  }
  generated.push_str("];\n");
  writeln!(
    generated,
    "const SOLID_PLUGIN_DEPENDENCY: &str = {};",
    dependency("eslint-plugin-solid")
  )
  .unwrap();

  let output_path = PathBuf::from(out_dir).join("dependencies.rs");
  write(&output_path, generated)
    .unwrap_or_else(|error| panic!("could not write {}: {error}", output_path.display()));

  for package_path in package_paths {
    println!("cargo:rerun-if-changed={}", package_path.display());
  }
}
