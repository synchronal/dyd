use crate::app::AppResult;
use std::path::{Path, PathBuf};
use std::process::Command;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone_repo(origin: &String, path: &Path) {
  let path_str = path.to_path_buf();

  Command::new("git")
    .args(["clone", origin, path_str.to_str().unwrap()])
    .output()
    .unwrap();
}

pub fn logs(path: &PathBuf, branch: &Option<String>) -> AppResult<Vec<u8>> {
  let mut logs = Command::new("git");
  logs
    .args([
      "log",
      "--date=local",
      "-n",
      "400",
      "--abbrev-commit",
      "--color=always",
      &format!("--pretty=tformat:{GIT_FORMAT}"),
    ])
    .current_dir(path);

  if let Some(branch) = branch {
    logs.arg(format!("origin/{branch}"));
  }

  Ok(logs.output()?.stdout)
}

pub fn pull_repo(path: &PathBuf) {
  Command::new("git")
    .args(["pull", "--prune"])
    .current_dir(path)
    .output()
    .unwrap();
}

pub fn switch_branch(path: &PathBuf, branch: String) {
  Command::new("git")
    .args(["switch", &branch])
    .current_dir(path)
    .output()
    .unwrap();
}
