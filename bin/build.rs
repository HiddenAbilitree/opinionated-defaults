use std::{
  collections::BTreeMap,
  env::var,
  fs::{copy, read_to_string, write},
  path::PathBuf,
};

use serde_json::Value;

fn main() {
  let out_dir = var("OUT_DIR").unwrap();
  let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();

  for name in &["oxlintrc.json", "oxfmtrc.json"] {
    let repo_src = format!("{manifest_dir}/../src/{name}");
    let local_src = format!("{manifest_dir}/src/{name}");
    let dst = format!("{out_dir}/{name}");

    if copy(&repo_src, &dst).is_err() {
      copy(&local_src, &dst)
        .unwrap_or_else(|_| panic!("could not find {name} at {repo_src} or {local_src}"));
    }

    println!("cargo:rerun-if-changed={repo_src}");
    println!("cargo:rerun-if-changed={local_src}");
  }

  generate_eslint_prettier_dependencies(&manifest_dir, &out_dir);
}

fn generate_eslint_prettier_dependencies(manifest_dir: &str, out_dir: &str) {
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

  let dependencies = dependencies
    .iter()
    .map(|(name, version)| {
      let version = version
        .as_str()
        .unwrap_or_else(|| panic!("peer dependency {name} must have a string version"));

      (name, format!("{name}@{version}"))
    })
    .collect::<BTreeMap<_, _>>();

  let mut generated = String::from("const ESLINT_PRETTIER_DEPENDENCIES: &[&str] = &[\n");
  for dependency in dependencies.values() {
    generated.push_str("  ");
    generated.push_str(
      &serde_json::to_string(dependency)
        .unwrap_or_else(|error| panic!("could not encode dependency {dependency}: {error}")),
    );
    generated.push_str(",\n");
  }
  generated.push_str("];\n");

  let output_path = PathBuf::from(out_dir).join("eslint_prettier_dependencies.rs");
  write(&output_path, generated)
    .unwrap_or_else(|error| panic!("could not write {}: {error}", output_path.display()));

  for package_path in package_paths {
    println!("cargo:rerun-if-changed={}", package_path.display());
  }
}
