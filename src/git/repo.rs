use crate::app::{AppResult, Event};
use crate::git;
use crate::manifest::Remote;
use crate::semaphore::Semaphore;
use crate::time;

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;

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
    let mut parts = log_str.split('\x0B');

    if let (Some(sha), Some(cdate), Some(age), Some(author), Some(message)) =
      (parts.next(), parts.next(), parts.next(), parts.next(), parts.next())
    {
      let commit_datetime = time::parse_unix(cdate).unwrap_or_else(|_| time::parse_unix("0").unwrap());
      Self {
        sha: sha.to_string(),
        cdate: cdate.to_string(),
        age: age.to_string(),
        author: author.to_string(),
        message: message.to_string(),
        commit_datetime,
      }
    } else {
      Self {
        sha: String::new(),
        cdate: String::new(),
        age: String::new(),
        author: String::new(),
        message: String::new(),
        commit_datetime: time::parse_unix("0").unwrap(),
      }
    }
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

#[cfg(test)]
mod tests {
  use super::*;
  // Import only what we need for tests

  #[test]
  fn test_log_parsing_valid_input() {
    let log_str = "a1b2c3d\x0B1640995200\x0B2 hours ago\x0BJohn Doe\x0BAdd new feature";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "a1b2c3d");
    assert_eq!(log.cdate, "1640995200");
    assert_eq!(log.age, "2 hours ago");
    assert_eq!(log.author, "John Doe");
    assert_eq!(log.message, "Add new feature");
    // Verify commit_datetime is parsed correctly
    assert_eq!(log.commit_datetime.timestamp(), 1640995200);
  }

  #[test]
  fn test_log_parsing_empty_input() {
    let log_str = "";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "");
    assert_eq!(log.cdate, "");
    assert_eq!(log.age, "");
    assert_eq!(log.author, "");
    assert_eq!(log.message, "");
    // Verify fallback datetime (unix timestamp 0)
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_incomplete_input() {
    let log_str = "a1b2c3d\x0B1640995200\x0B2 hours ago";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "");
    assert_eq!(log.cdate, "");
    assert_eq!(log.age, "");
    assert_eq!(log.author, "");
    assert_eq!(log.message, "");
    // Verify fallback datetime when parsing fails
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_with_empty_fields() {
    let log_str = "\x0B\x0B\x0B\x0B";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "");
    assert_eq!(log.cdate, "");
    assert_eq!(log.age, "");
    assert_eq!(log.author, "");
    assert_eq!(log.message, "");
    // Empty cdate should fallback to timestamp 0
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_with_invalid_timestamp() {
    let log_str = "abc123\x0Binvalid_timestamp\x0B3 days ago\x0BAlice Smith\x0BFix bug";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "abc123");
    assert_eq!(log.cdate, "invalid_timestamp");
    assert_eq!(log.age, "3 days ago");
    assert_eq!(log.author, "Alice Smith");
    assert_eq!(log.message, "Fix bug");
    // Invalid timestamp should fallback to 0
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_with_negative_timestamp() {
    let log_str = "def456\x0B-1\x0B1 week ago\x0BBob Jones\x0BInitial commit";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "def456");
    assert_eq!(log.cdate, "-1");
    assert_eq!(log.age, "1 week ago");
    assert_eq!(log.author, "Bob Jones");
    assert_eq!(log.message, "Initial commit");
    // Negative timestamp should be handled correctly
    assert_eq!(log.commit_datetime.timestamp(), -1);
  }

  #[test]
  fn test_log_parsing_with_special_characters_in_message() {
    // Test with quotes, newlines, but avoiding \x0B in message since it's the separator
    let log_str = "ghi789\x0B1650000000\x0B1 day ago\x0BTest User\x0BMessage with \"quotes\" and \nnewlines";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "ghi789");
    assert_eq!(log.cdate, "1650000000");
    assert_eq!(log.age, "1 day ago");
    assert_eq!(log.author, "Test User");
    assert_eq!(log.message, "Message with \"quotes\" and \nnewlines");
    assert_eq!(log.commit_datetime.timestamp(), 1650000000);
  }

  #[test]
  fn test_log_parsing_separator_in_message_field() {
    // This test demonstrates the current parsing limitation:
    // If the message contains the separator \x0B, it will be split incorrectly
    let log_str = "problematic\x0B1650000000\x0B1 day ago\x0BTest User\x0BMessage with \x0B separator";
    let log: Log = log_str.into();

    // Due to split() behavior, everything after the \x0B in message becomes parts[5] and beyond
    // The message field will only contain the part before the separator
    assert_eq!(log.sha, "problematic");
    assert_eq!(log.cdate, "1650000000");
    assert_eq!(log.age, "1 day ago");
    assert_eq!(log.author, "Test User");
    assert_eq!(log.message, "Message with "); // Only the part before \x0B
    assert_eq!(log.commit_datetime.timestamp(), 1650000000);
  }

  #[test]
  fn test_log_parsing_with_unicode_characters() {
    let log_str = "unicode\x0B1640995200\x0B2时间前\x0B张三\x0B添加新功能 🚀";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "unicode");
    assert_eq!(log.cdate, "1640995200");
    assert_eq!(log.age, "2时间前");
    assert_eq!(log.author, "张三");
    assert_eq!(log.message, "添加新功能 🚀");
    assert_eq!(log.commit_datetime.timestamp(), 1640995200);
  }

  #[test]
  fn test_log_parsing_extra_fields_ignored() {
    let log_str = "extra\x0B1640995200\x0B5 min ago\x0BDev User\x0BUpdate docs\x0Bextra_field\x0Banother_extra";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "extra");
    assert_eq!(log.cdate, "1640995200");
    assert_eq!(log.age, "5 min ago");
    assert_eq!(log.author, "Dev User");
    assert_eq!(log.message, "Update docs");
    // Extra fields should be ignored, processing should succeed
    assert_eq!(log.commit_datetime.timestamp(), 1640995200);
  }

  #[test]
  fn test_log_parsing_boundary_timestamps() {
    // Test with maximum valid Unix timestamp (Year 2038 problem boundary)
    let log_str = "boundary\x0B2147483647\x0Bfar future\x0BTime Traveler\x0BFuture commit";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "boundary");
    assert_eq!(log.cdate, "2147483647");
    assert_eq!(log.commit_datetime.timestamp(), 2147483647);
  }

  #[test]
  fn test_log_parsing_zero_timestamp() {
    let log_str = "epoch\x0B0\x0Bepoch\x0BEpoch User\x0BEpoch commit";
    let log: Log = log_str.into();

    assert_eq!(log.sha, "epoch");
    assert_eq!(log.cdate, "0");
    assert_eq!(log.age, "epoch");
    assert_eq!(log.author, "Epoch User");
    assert_eq!(log.message, "Epoch commit");
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_whitespace_fields() {
    let log_str = "   \x0B  1640995200  \x0B  2 hours ago  \x0B  John Doe  \x0B  Add feature  ";
    let log: Log = log_str.into();

    // Whitespace should be preserved as-is (not trimmed)
    assert_eq!(log.sha, "   ");
    assert_eq!(log.cdate, "  1640995200  ");
    assert_eq!(log.age, "  2 hours ago  ");
    assert_eq!(log.author, "  John Doe  ");
    assert_eq!(log.message, "  Add feature  ");
    // time::parse_unix will fail on "  1640995200  " (whitespace), fallback to 0
    assert_eq!(log.commit_datetime.timestamp(), 0);
  }

  #[test]
  fn test_log_parsing_performance_critical_path() {
    // Test the most common case that will be hit in production
    let typical_log =
      "f8a3b2c1d4e5f678\x0B1672531200\x0B3 hours ago\x0Bjdoe@example.com\x0Bfeat: add user authentication";
    let log: Log = typical_log.into();

    assert_eq!(log.sha, "f8a3b2c1d4e5f678");
    assert_eq!(log.cdate, "1672531200");
    assert_eq!(log.age, "3 hours ago");
    assert_eq!(log.author, "jdoe@example.com");
    assert_eq!(log.message, "feat: add user authentication");
    assert_eq!(log.commit_datetime.timestamp(), 1672531200);
  }
}
