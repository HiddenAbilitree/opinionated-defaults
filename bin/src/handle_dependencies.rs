use {crate::types::Dependencies, log::info, std::collections::BTreeSet};

pub fn handle_dependencies(deps: Dependencies) -> Vec<String> {
  let mut present: BTreeSet<String> = deps.default_deps.into_iter().collect();

  for (pkg_name, import_name) in deps.valid_deps {
    if deps.packages.contains_key(&pkg_name) {
      info!("Found dependency {pkg_name}");
      present.insert(import_name);
    }
  }

  present.into_iter().collect()
}
