use crate::difftool::Difftool;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Manifest {
  #[serde(default = "default_difftool")]
  pub(crate) difftool: Difftool,
  pub(crate) since: String,
  pub(crate) remotes: HashMap<String, Remote>,
}

impl Manifest {
  pub fn new(path: PathBuf) -> Result<Manifest> {
    let manifest_contents =
      std::fs::read_to_string(&path).with_context(|| format!("Error reading file: `{}`", path.to_str().unwrap()))?;

    Ok(toml::from_str(&manifest_contents)?)
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
