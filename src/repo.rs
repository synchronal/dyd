use crate::app::AppResult;
use crate::event::Event;
use crate::manifest::Remote;

use std::path::{Path, PathBuf};
use std::process::Command;
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

impl Repo {
    pub fn update(&self, id: String, root_path: &PathBuf, sender: mpsc::Sender<Event>) -> AppResult<()> {
        let path = self.path(root_path)?;
        let path_str = path.clone();
        let origin = self.origin.clone();

        std::thread::spawn(move || {
            if path.is_dir() {
                sender
                    .send(Event::RepoStatusChange(id.clone(), RepoStatus::Pulling))
                    .unwrap();
                Command::new("git")
                    .args(["pull", &origin, path_str.to_str().unwrap()])
                    .output()
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                sender
                    .send(Event::RepoStatusChange(id.clone(), RepoStatus::Cloning))
                    .unwrap();
                Command::new("git")
                    .args(["clone", &origin, path_str.to_str().unwrap()])
                    .output()
                    .unwrap();
            }
            sender
                .send(Event::RepoStatusChange(id.clone(), RepoStatus::Log))
                .unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));

            sender
                .send(Event::RepoStatusChange(id.clone(), RepoStatus::Finished))
                .unwrap();
            ()
        });
        Ok(())
    }

    fn path(&self, root: &PathBuf) -> AppResult<PathBuf> {
        if let Some(path) = Path::new(&self.origin).file_name() {
            Ok(root.join(path))
        } else {
            Err(format!("Unable to determine local path for {}", self.name).into())
        }
    }
}
