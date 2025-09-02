use crate::app::AppResult;
use gix::remote::Direction;
use log;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

pub mod repo;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone_repo(origin: &str, path: &Path) -> Result<(), Box<dyn Error>> {
  log::info!("starting git clone: remote: \"{origin}\", path: {path:?}");
  std::fs::create_dir_all(path)?;
  let mut prepare_clone = gix::prepare_clone(origin, path)?;

  let (mut prepare_checkout, _) =
    prepare_clone.fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

  let (repo, _) = prepare_checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
  let _remote = repo
    .find_default_remote(Direction::Fetch)
    .expect("always present after clone")?;

  log::info!("finished git clone: remote: \"{origin}\", path: {path:?}");

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

pub fn pull_repo(path: &PathBuf) -> AppResult<()> {
  log::info!("starting git fetch: {path:?}");
  let repo = gix::discover(path)?;
  log::debug!("repo: {repo:?}");
  let head = repo.head()?;
  let remote = match head.into_remote(Direction::Fetch) {
    Some(r) => r?,
    None => {
      log::error!("failed git fetch: {path:?}, reason: \"unable to find remote\"");
      log::debug!("failed repo: {repo:?}");
      return Err("Unable to fetch remote".into());
    }
  };

  remote
    .connect(Direction::Fetch)?
    .prepare_fetch(gix::progress::Discard, Default::default())?
    .receive(gix::progress::Discard, &AtomicBool::default())?;

  log::info!("finished git fetch: {path:?}");
  log::info!("starting git merge: {path:?}");

  Command::new("git")
    .args(["merge", "--no-edit", "--ff-only", "--quiet", "--no-commit"])
    .current_dir(path)
    .output()
    .unwrap();

  log::info!("finished git merge: {path:?}");

  Ok(())
}

pub fn switch_branch(path: &PathBuf, branch: &str) {
  Command::new("git")
    .args(["switch", branch])
    .current_dir(path)
    .output()
    .unwrap();
}
