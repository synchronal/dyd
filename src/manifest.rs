use crate::cli::CLI;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    since: String,
    remotes: HashMap<String, Remote>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            since: "1 week ago".to_string(),
            remotes: HashMap::new(),
        }
    }
}

impl Manifest {
    pub fn new(args: CLI) -> Result<Manifest, Box<dyn std::error::Error>> {
        let manifest_contents = std::fs::read_to_string(&args.manifest).with_context(|| {
            format!("Error reading file: `{}`", &args.manifest.to_str().unwrap())
        })?;

        let manifest: Manifest = toml::from_str(&manifest_contents)?;
        Ok(manifest)
    }
}

#[derive(Debug, Deserialize)]
pub struct Remote {
    name: String,
    origin: String,
}
