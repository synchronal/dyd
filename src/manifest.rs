use crate::cli::CLI;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub(crate) since: String,
    pub(crate) remotes: HashMap<String, Remote>,
    pub(crate) root: Option<PathBuf>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            since: "1 week ago".to_string(),
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
        manifest.root = Some(root);
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Remote {
    pub(crate) name: String,
    pub(crate) origin: String,
}
