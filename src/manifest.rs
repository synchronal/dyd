use crate::difftool::Difftool;
use crate::time;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Manifest {
  #[serde(default = "default_difftool")]
  pub(crate) difftool: Difftool,
  pub(crate) since: String,
  #[serde(skip)]
  pub(crate) since_datetime: Option<chrono::DateTime<chrono::Utc>>,
  pub(crate) remotes: HashMap<String, Remote>,
  pub(crate) root: Option<PathBuf>,
}

impl Manifest {
  pub fn new(path: std::path::PathBuf, root: PathBuf) -> Result<Manifest, Box<dyn std::error::Error>> {
    let manifest_contents =
      std::fs::read_to_string(&path).with_context(|| format!("Error reading file: `{}`", path.to_str().unwrap()))?;

    let mut manifest: Manifest = toml::from_str(&manifest_contents)?;
    let since_datetime = time::parse_relative(&manifest.since, &chrono::Utc::now())?;
    manifest.root = Some(root);
    manifest.since_datetime = Some(since_datetime);
    Ok(manifest)
  }
}

#[derive(Debug, Deserialize)]
pub struct Remote {
  pub(crate) name: String,
  pub(crate) origin: String,
  pub(crate) branch: Option<String>,
}

fn default_difftool() -> Difftool {
  Difftool::Git
}
