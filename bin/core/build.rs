use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
  let version = resolve_version()
    .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap());
  println!("cargo::rustc-env=KOMODO_VERSION={version}");

  let root = workspace_root();
  println!("cargo::rerun-if-env-changed=KOMODO_BUILD_VERSION");
  println!(
    "cargo::rerun-if-changed={}",
    root.join(".git/HEAD").display()
  );
  if let Ok(head) = std::fs::read_to_string(root.join(".git/HEAD")) {
    if let Some(ref_path) = head.strip_prefix("ref: ").map(str::trim)
    {
      println!(
        "cargo::rerun-if-changed={}",
        root.join(".git").join(ref_path).display()
      );
    }
  }
}

fn resolve_version() -> Option<String> {
  if let Ok(v) = std::env::var("KOMODO_BUILD_VERSION") {
    let v = v.trim();
    if !v.is_empty() {
      return Some(v.to_string());
    }
  }
  git_short_hash(&workspace_root())
}

fn git_short_hash(dir: &Path) -> Option<String> {
  let out = Command::new("git")
    .args(["rev-parse", "--short", "HEAD"])
    .current_dir(dir)
    .output()
    .ok()?;
  if !out.status.success() {
    return None;
  }
  let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
  if v.is_empty() { None } else { Some(v) }
}

fn workspace_root() -> PathBuf {
  let manifest_dir =
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
  manifest_dir
    .parent()
    .and_then(Path::parent)
    .map(PathBuf::from)
    .unwrap_or(manifest_dir)
}
