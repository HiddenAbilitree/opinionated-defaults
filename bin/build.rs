fn main() {
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

  for name in &["oxlintrc.json", "oxfmtrc.json"] {
    let repo_src = format!("{manifest_dir}/../src/{name}");
    let local_src = format!("{manifest_dir}/src/{name}");
    let dst = format!("{out_dir}/{name}");

    if std::fs::copy(&repo_src, &dst).is_err() {
      std::fs::copy(&local_src, &dst)
        .unwrap_or_else(|_| panic!("could not find {name} at {repo_src} or {local_src}"));
    }

    println!("cargo:rerun-if-changed={repo_src}");
    println!("cargo:rerun-if-changed={local_src}");
  }
}
