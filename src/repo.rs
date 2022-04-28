use crate::app::AppResult;
use crate::event::Event;
use crate::git;
use crate::manifest::Remote;
use crate::time;

use std::path::{Path, PathBuf};
use std::sync::mpsc;

#[derive(Clone, Debug)]
pub enum RepoStatus {
    Checking,
    Cloning,
    Pulling,
    Failed,
    Log,
    Finished,
}

impl Default for RepoStatus {
    fn default() -> Self {
        RepoStatus::Checking
    }
}

#[derive(Debug, Default)]
pub struct Repo {
    pub(crate) logs: Vec<Log>,
    pub(crate) name: String,
    pub(crate) origin: String,
    pub(crate) status: RepoStatus,
}

impl From<Remote> for Repo {
    fn from(remote: Remote) -> Self {
        Repo {
            name: remote.name,
            origin: remote.origin,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct Log {
    pub(crate) age: String,
    pub(crate) author: String,
    pub commit_datetime: chrono::DateTime<chrono::Utc>,
    pub(crate) message: String,
    pub(crate) sha: String,
}

impl From<&str> for Log {
    fn from(log_str: &str) -> Self {
        let values: Vec<&str> = log_str.split("\x0B").collect();
        let sha = values[0].to_owned();
        let commit_datetime = time::parse_unix(&values[1].to_owned()).unwrap();
        let age = values[2].to_owned();
        let author = values[3].to_owned();
        let message = values[4].to_owned();
        Self {
            age,
            author,
            commit_datetime,
            message,
            sha,
        }
    }
}

impl Repo {
    pub fn update(&self, id: String, root_path: &PathBuf, sender: mpsc::Sender<Event>) -> AppResult<()> {
        let path = self.path(root_path)?;
        let origin = self.origin.clone();

        std::thread::spawn(move || {
            if path.is_dir() {
                sender
                    .send(Event::RepoStatusChange(id.clone(), RepoStatus::Pulling))
                    .unwrap();

                git::pull(&path);
            } else {
                sender
                    .send(Event::RepoStatusChange(id.clone(), RepoStatus::Cloning))
                    .unwrap();
                git::clone(&origin, &path);
            }
            sender
                .send(Event::RepoStatusChange(id.clone(), RepoStatus::Log))
                .unwrap();

            let logs = Repo::logs(&path);
            sender
                .send(Event::RepoStatusComplete(id.clone(), logs))
                .unwrap();

            ()
        });
        Ok(())
    }

    pub fn path(&self, root: &PathBuf) -> AppResult<PathBuf> {
        if let Some(path) = Path::new(&self.origin).file_name() {
            Ok(root.join(path))
        } else {
            Err(format!("Unable to determine local path for {}", self.name).into())
        }
    }

    fn logs(path: &PathBuf) -> Vec<Log> {
        let logs = git::logs(path);

        std::str::from_utf8(&logs)
            .unwrap()
            .trim()
            .split("\n")
            .into_iter()
            .map(|l| l.into())
            .collect()
    }
}
