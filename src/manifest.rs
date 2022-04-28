use crate::cli::CLI;
use crate::time;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub(crate) since: String,
    #[serde(skip)]
    pub(crate) since_datetime: Option<chrono::DateTime<chrono::Utc>>,
    pub(crate) remotes: HashMap<String, Remote>,
    pub(crate) root: Option<PathBuf>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            since: "1 week ago".to_string(),
            since_datetime: None,
            remotes: HashMap::new(),
            root: None,
        }
    }
}

impl Manifest {
    pub fn new(args: CLI, root: PathBuf) -> Result<Manifest, Box<dyn std::error::Error>> {
        let manifest_contents = std::fs::read_to_string(&args.manifest)
            .with_context(|| format!("Error reading file: `{}`", &args.manifest.to_str().unwrap()))?;

        let mut manifest: Manifest = toml::from_str(&manifest_contents)?;
        let since_datetime = time::parse_relative(&manifest.since, &chrono::Utc::now())?;
        manifest.root = Some(root);
        manifest.since_datetime = Some(since_datetime);
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Remote {
    pub(crate) name: String,
    pub(crate) origin: String,
}
