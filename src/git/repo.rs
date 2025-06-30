use crate::app::{AppResult, Event};
use crate::git;
use crate::manifest::Remote;
use crate::time;

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd)]
pub enum RepoStatus {
  #[default]
  Checking,
  Cloning,
  Pulling,
  Failed,
  Log,
  Finished,
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
  pub age: String,
  pub author: String,
  pub cdate: String,
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

impl From<&str> for Log {
  fn from(log_str: &str) -> Self {
    let values: Vec<&str> = log_str.split('\x0B').collect();

    if let &[sha, cdate, age, author, message] = &*values {
      let sha = sha.to_owned();
      let commit_datetime = time::parse_unix(cdate).unwrap();
      let age = age.to_owned();
      let author = author.to_owned();
      let message = message.to_owned();
      Self {
        age,
        author,
        cdate: cdate.to_owned(),
        commit_datetime,
        message,
        sha,
      }
    } else {
      Self {
        age: "".to_owned(),
        author: "".to_owned(),
        cdate: "".to_owned(),
        commit_datetime: time::parse_unix("0").unwrap(),
        message: "".to_owned(),
        sha: "".to_owned(),
      }
    }
  }
}

impl Repo {
  pub fn update(&self, id: String, root_path: &Path, sender: mpsc::Sender<Event>) -> AppResult<()> {
    let path = self.path(root_path)?;
    let origin = self.origin.clone();
    let branch = self.branch.clone();

    std::thread::spawn(move || {
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

        if let Err(err) = git::clone_repo(origin, &path) {
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

      if let Ok(logs) = Repo::logs(&path, &branch) {
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

  fn logs(path: &PathBuf, branch: &Option<String>) -> AppResult<Vec<Log>> {
    let logs = git::logs(path, branch)?;

    Ok(
      std::str::from_utf8(&logs)
        .unwrap()
        .trim()
        .split('\n')
        .map(|l| l.into())
        .collect(),
    )
  }
}
