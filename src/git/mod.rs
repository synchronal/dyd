use crate::app::AppResult;
use gix::remote::Direction;
use log;
use std::error::Error;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicBool;

pub mod repo;

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

pub fn logs(path: &Path, branch: Option<&str>) -> AppResult<Vec<repo::Log>> {
  let repo = gix::discover(path)?;

  let tip = match branch {
    Some(b) => repo
      .find_reference(&format!("refs/remotes/origin/{b}"))?
      .into_fully_peeled_id()?
      .detach(),
    None => repo.head_id()?.detach(),
  };

  let mut logs = Vec::new();

  for info in repo.rev_walk([tip]).all()?.take(400) {
    let info = info?;
    let commit = repo.find_commit(info.id)?;
    let short_id = commit.short_id()?.to_string();
    let author = commit.author()?.name.to_string();
    let seconds = commit.time()?.seconds;
    let commit_datetime =
      chrono::DateTime::from_timestamp(seconds, 0).unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    let message = commit.message()?.title.to_string();

    logs.push(repo::Log {
      author,
      commit_datetime,
      message,
      sha: short_id,
    });
  }

  Ok(logs)
}

pub fn pull_repo(path: &Path) -> AppResult<()> {
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

pub fn switch_branch(path: &Path, branch: &str) {
  Command::new("git")
    .args(["switch", branch])
    .current_dir(path)
    .output()
    .unwrap();
}
