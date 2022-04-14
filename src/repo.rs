use crate::manifest::Remote;

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
