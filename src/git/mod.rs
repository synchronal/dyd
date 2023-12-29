use crate::app::AppResult;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod repo;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone_repo(origin: String, path: &Path) -> Result<(), Box<dyn Error>> {
  gix::interrupt::init_handler(1, || {})?;
  std::fs::create_dir_all(path)?;
  let mut prepare_clone = gix::prepare_clone(origin, path)?;

  let (mut prepare_checkout, _) =
    prepare_clone.fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

  let (repo, _) = prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
  let _remote = repo
    .find_default_remote(gix::remote::Direction::Fetch)
    .expect("always present after clone")?;

  Ok(())
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
