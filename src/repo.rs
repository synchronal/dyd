use crate::app::AppResult;
use crate::manifest::Remote;

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
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
    pub fn update(&self, _id: &String, root_path: &PathBuf) -> AppResult<()> {
        let path = self.path(root_path)?;
        let path_str = path.clone();
        let origin = self.origin.clone();

        // std::thread::spawn(move || {
        if path.is_dir() {
            Command::new("git")
                .args(["clone", &origin, path_str.to_str().unwrap()])
                .output()?;
        } else {
            Command::new("git")
                .args(["pull", &origin, path_str.to_str().unwrap()])
                .output()?;
        }
        // ()
        // });
        Ok(())
    }

    fn path(&self, root: &PathBuf) -> AppResult<PathBuf> {
        if let Some(path) = Path::new(&self.origin).file_name() {
            Ok(root.join(path))
            // Err(format!("Unable to determine local path for {}", self.name).into())
        } else {
            Err(format!("Unable to determine local path for {}", self.name).into())
        }
    }
}
