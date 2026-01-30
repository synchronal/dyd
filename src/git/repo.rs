use crate::app::{AppResult, Event};
use crate::git;
use crate::manifest::Remote;
use crate::semaphore::Semaphore;

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
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

impl Ord for Repo {
  fn cmp(&self, other: &Self) -> Ordering {
    if !self.logs.is_empty() && !other.logs.is_empty() {
      return self.logs[0].cmp(&other.logs[0]);
    };

    if self.logs.is_empty() && !other.logs.is_empty() {
      return Ordering::Greater;
    }
    if !self.logs.is_empty() && other.logs.is_empty() {
      return Ordering::Less;
    }

    if self.name > other.name {
      return Ordering::Greater;
    }
    Ordering::Less
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

impl Ord for Log {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.commit_datetime > other.commit_datetime {
      return Ordering::Less;
    };
    if self.commit_datetime < other.commit_datetime {
      return Ordering::Greater;
    };
    if self.message > other.message {
      return Ordering::Less;
    }
    Ordering::Greater
  }
}

impl PartialOrd for Log {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Repo {
  pub fn update(
    &self,
    id: &str,
    root_path: &Path,
    sender: mpsc::Sender<Event>,
    semaphore: Arc<Semaphore>,
  ) -> AppResult<()> {
    let path = self.path(root_path)?;
    let origin = self.origin.clone();
    let branch = self.branch.clone();
    let id = id.to_string();

    std::thread::spawn(move || {
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
}
