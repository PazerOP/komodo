use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
  let version = resolve_version()
    .unwrap_or_else(|| std::env::var("CARGO_PKG_VERSION").unwrap());
  println!("cargo::rustc-env=KOMODO_VERSION={version}");

  println!("cargo::rerun-if-env-changed=KOMODO_BUILD_VERSION");
  let root = workspace_root();
  println!(
    "cargo::rerun-if-changed={}",
    root.join("build_support/version.rs").display()
  );
  println!(
    "cargo::rerun-if-changed={}",
    root.join("build_support/BUILD_VERSION").display()
  );
  emit_git_rerun_if_changed(&root);
}

fn resolve_version() -> Option<String> {
  if let Ok(v) = std::env::var("KOMODO_BUILD_VERSION") {
    let v = v.trim();
    if !v.is_empty() {
      return Some(v.to_string());
    }
  }
  let root = workspace_root();
  if let Ok(v) = std::fs::read_to_string(root.join("build_support/BUILD_VERSION")) {
    let v = v.trim();
    if !v.is_empty() {
      return Some(v.to_string());
    }
  }
  git_short_hash(&root)
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

fn emit_git_rerun_if_changed(root: &Path) {
  let Some(git_dir) = git_dir(root) else {
    return;
  };
  let head_path = git_dir.join("HEAD");
  if !head_path.exists() {
    return;
  }
  println!("cargo::rerun-if-changed={}", head_path.display());
  if let Ok(head) = std::fs::read_to_string(&head_path) {
    if let Some(ref_path) = head.strip_prefix("ref: ").map(str::trim) {
      let ref_file = git_dir.join(ref_path);
      if ref_file.exists() {
        println!("cargo::rerun-if-changed={}", ref_file.display());
      }
    }
  }
}

fn git_dir(root: &Path) -> Option<PathBuf> {
  let dot_git = root.join(".git");
  let metadata = std::fs::metadata(&dot_git).ok()?;
  if metadata.is_dir() {
    return Some(dot_git);
  }
  if !metadata.is_file() {
    return None;
  }
  let contents = std::fs::read_to_string(&dot_git).ok()?;
  let gitdir = contents.strip_prefix("gitdir: ")?.trim();
  let git_dir = Path::new(gitdir);
  Some(if git_dir.is_absolute() {
    git_dir.to_path_buf()
  } else {
    dot_git.parent().unwrap_or(root).join(git_dir)
  })
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
