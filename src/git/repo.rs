use crate::app::{AppResult, Event};
use crate::git;
use crate::manifest::Remote;
use crate::refresh::RefreshTask;
use crate::semaphore::Semaphore;

use std::cmp::Ordering;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum RepoStatus {
  #[default]
  Checking,
  Cloning,
  Failed,
  Finished,
  Log,
  Pulling,
}

impl std::fmt::Display for RepoStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      RepoStatus::Checking => write!(f, " ⁇"),
      RepoStatus::Cloning => write!(f, " ⚭"),
      RepoStatus::Failed => write!(f, " 𝗫"),
      RepoStatus::Finished => write!(f, " ✓"),
      RepoStatus::Log => write!(f, " ☈"),
      RepoStatus::Pulling => write!(f, " ⤵"),
    }
  }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Repo {
  pub(crate) branch: Option<String>,
  pub(crate) logs: Vec<Log>,
  pub(crate) name: String,
  pub(crate) origin: String,
  pub(crate) status: RepoStatus,
}

impl std::fmt::Display for Repo {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name)?;
    if let Some(branch) = &self.branch {
      write!(f, " — ({branch})")
    } else {
      Ok(())
    }
  }
}

impl From<Remote> for Repo {
  fn from(remote: Remote) -> Self {
    Repo {
      branch: remote.branch,
      name: remote.name,
      origin: remote.origin,
      ..Default::default()
    }
  }
}

/// Newest commit first, then repos without logs, by name. Remaining fields
/// only break ties, keeping `cmp` consistent with `Eq`.
impl Ord for Repo {
  fn cmp(&self, other: &Self) -> Ordering {
    match (self.logs.first(), other.logs.first()) {
      (Some(log), Some(other_log)) => log.cmp(other_log),
      (Some(_), None) => Ordering::Less,
      (None, Some(_)) => Ordering::Greater,
      (None, None) => Ordering::Equal,
    }
    .then_with(|| self.name.cmp(&other.name))
    .then_with(|| self.origin.cmp(&other.origin))
    .then_with(|| self.branch.cmp(&other.branch))
    .then_with(|| self.logs.cmp(&other.logs))
    .then_with(|| self.status.cmp(&other.status))
  }
}

impl PartialOrd for Repo {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Log {
  pub author: String,
  pub commit_datetime: chrono::DateTime<chrono::Utc>,
  pub message: String,
  pub sha: String,
}

/// Newest commit first, then by message descending. Remaining fields only
/// break ties, keeping `cmp` consistent with `Eq`.
impl Ord for Log {
  fn cmp(&self, other: &Self) -> Ordering {
    other
      .commit_datetime
      .cmp(&self.commit_datetime)
      .then_with(|| other.message.cmp(&self.message))
      .then_with(|| self.sha.cmp(&other.sha))
      .then_with(|| self.author.cmp(&other.author))
  }
}

impl PartialOrd for Log {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl TryFrom<gix::Commit<'_>> for Log {
  type Error = Box<dyn Error>;

  fn try_from(commit: gix::Commit<'_>) -> Result<Self, Self::Error> {
    let sha = commit.short_id()?.to_string();
    let author = commit.author()?.name.to_string();
    let seconds = commit.time()?.seconds;
    let commit_datetime =
      chrono::DateTime::from_timestamp(seconds, 0).unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
    let message = commit.message()?.title.to_string();

    Ok(Log {
      author,
      commit_datetime,
      message,
      sha,
    })
  }
}

impl Repo {
  pub fn update(
    &self,
    id: &str,
    root_path: &Path,
    sender: mpsc::Sender<Event>,
    semaphore: Arc<Semaphore>,
    task: RefreshTask,
  ) -> AppResult<()> {
    let path = self.path(root_path)?;
    let origin = self.origin.clone();
    let branch = self.branch.clone();
    let id = id.to_string();

    std::thread::spawn(move || {
      let _task = task;
      let _permit = semaphore.acquire();

      if path.is_dir() {
        sender
          .send(Event::RepoStatusChange(id.clone(), RepoStatus::Pulling))
          .unwrap();

        if let Err(err) = git::pull_repo(&path) {
          log::error!("failed git pull: {path:?}, reason: {err}");
          sender
            .send(Event::RepoStatusChange(id.clone(), RepoStatus::Failed))
            .unwrap();

          return;
        }
      } else {
        sender
          .send(Event::RepoStatusChange(id.clone(), RepoStatus::Cloning))
          .unwrap();

        if let Err(err) = git::clone_repo(&origin, &path) {
          log::error!("failed git clone: {path:?}, reason: {err}");
          sender
            .send(Event::RepoStatusChange(id.clone(), RepoStatus::Failed))
            .unwrap();

          return;
        }
      }
      sender
        .send(Event::RepoStatusChange(id.clone(), RepoStatus::Log))
        .unwrap();

      if let Ok(logs) = Repo::logs(&path, branch.as_deref()) {
        sender
          .send(Event::RepoStatusComplete(id.clone(), logs))
          .unwrap();
      };
    });
    Ok(())
  }

  pub fn path(&self, root: &Path) -> AppResult<PathBuf> {
    if let Some(path) = Path::new(&self.origin).file_name() {
      Ok(root.join(path))
    } else {
      Err(format!("Unable to determine local path for {}", self.name).into())
    }
  }

  fn logs(path: &Path, branch: Option<&str>) -> AppResult<Vec<Log>> {
    git::logs(path, branch)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_log_ordering_newer_first() {
    let newer = Log {
      author: "Alice".into(),
      commit_datetime: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
      message: "Second".into(),
      sha: "abc".into(),
    };
    let older = Log {
      author: "Bob".into(),
      commit_datetime: chrono::DateTime::from_timestamp(500, 0).unwrap(),
      message: "First".into(),
      sha: "def".into(),
    };

    assert_eq!(newer.cmp(&older), Ordering::Less);
    assert_eq!(older.cmp(&newer), Ordering::Greater);
  }

  #[test]
  fn test_log_ordering_same_time_sorts_by_message() {
    let log_a = Log {
      author: "Alice".into(),
      commit_datetime: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
      message: "BBB".into(),
      sha: "abc".into(),
    };
    let log_b = Log {
      author: "Bob".into(),
      commit_datetime: chrono::DateTime::from_timestamp(1000, 0).unwrap(),
      message: "AAA".into(),
      sha: "def".into(),
    };

    assert_eq!(log_a.cmp(&log_b), Ordering::Less);
    assert_eq!(log_b.cmp(&log_a), Ordering::Greater);
  }

  fn log(timestamp: i64, message: &str, sha: &str, author: &str) -> Log {
    Log {
      author: author.into(),
      commit_datetime: chrono::DateTime::from_timestamp(timestamp, 0).unwrap(),
      message: message.into(),
      sha: sha.into(),
    }
  }

  fn repo(name: &str, origin: &str, branch: Option<&str>, logs: Vec<Log>, status: RepoStatus) -> Repo {
    Repo {
      branch: branch.map(Into::into),
      logs,
      name: name.into(),
      origin: origin.into(),
      status,
    }
  }

  /// Checks consistency with `Eq`, antisymmetry and transitivity over every combination.
  fn assert_total_order<T: Ord + std::fmt::Debug>(values: &[T]) {
    for a in values {
      for b in values {
        assert_eq!(a.cmp(b) == Ordering::Equal, a == b, "{a:?} vs {b:?}");
        assert_eq!(a.cmp(b), b.cmp(a).reverse(), "{a:?} vs {b:?}");
        for c in values {
          if a <= b && b <= c {
            assert!(a <= c, "{a:?} <= {b:?} <= {c:?}");
          }
        }
      }
    }
  }

  fn logs() -> Vec<Log> {
    vec![
      log(1000, "same", "abc", "Alice"),
      log(1000, "same", "abc", "Bob"),
      log(1000, "same", "def", "Alice"),
      log(1000, "other", "abc", "Alice"),
      log(500, "same", "abc", "Alice"),
    ]
  }

  #[test]
  fn test_log_ordering_is_total() {
    assert_total_order(&logs());
  }

  #[test]
  fn test_repo_ordering_is_total() {
    let [newer, same_time, _, _, older] = <[Log; 5]>::try_from(logs()).unwrap();
    let repos = [
      repo("a", "git@x:a", None, vec![], RepoStatus::Checking),
      repo("a", "git@x:a", None, vec![], RepoStatus::Failed),
      repo("a", "git@x:a", Some("main"), vec![], RepoStatus::Checking),
      repo("a", "git@x:b", None, vec![], RepoStatus::Checking),
      repo("b", "git@x:a", None, vec![], RepoStatus::Checking),
      repo("a", "git@x:a", None, vec![newer.clone()], RepoStatus::Finished),
      repo(
        "a",
        "git@x:a",
        None,
        vec![newer.clone(), older.clone()],
        RepoStatus::Finished,
      ),
      repo("b", "git@x:a", None, vec![newer], RepoStatus::Finished),
      repo("a", "git@x:a", None, vec![same_time], RepoStatus::Finished),
      repo("a", "git@x:a", None, vec![older], RepoStatus::Finished),
    ];
    assert_total_order(&repos);
  }

  #[test]
  fn test_repo_ordering_newest_commit_first_then_unloaded_by_name() {
    let mut repos = [
      repo("unloaded-b", "git@x:ub", None, vec![], RepoStatus::Checking),
      repo(
        "older",
        "git@x:o",
        None,
        vec![log(500, "m", "abc", "A")],
        RepoStatus::Finished,
      ),
      repo("unloaded-a", "git@x:ua", None, vec![], RepoStatus::Checking),
      repo(
        "newer",
        "git@x:n",
        None,
        vec![log(1000, "m", "def", "A")],
        RepoStatus::Finished,
      ),
    ];
    repos.sort();

    let names: Vec<&str> = repos.iter().map(|repo| repo.name.as_str()).collect();
    assert_eq!(names, ["newer", "older", "unloaded-a", "unloaded-b"]);
  }
}
