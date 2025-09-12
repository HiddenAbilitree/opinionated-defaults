use {crate::types::Dependencies, anyhow::Result, log::info, std::collections::BTreeSet};

pub fn handle_dependencies(
  Dependencies {
    packages,
    valid_deps,
    default_deps,
  }: Dependencies,
) -> Result<Vec<String>> {
  let mut present_deps: BTreeSet<String> = default_deps.iter().map(|dep| dep.to_string()).collect();

  for dep in valid_deps {
    let (k, v) = dep;
    if packages.contains_key(&k) {
      info!("Found dependency {k}");
      present_deps.insert(v.to_string());
    }
  }

  let imports: Vec<String> = present_deps.iter().cloned().collect();

  Ok(imports)
}
