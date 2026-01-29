use crate::difftool::Difftool;
use crate::time;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
struct ManifestParseError(String);

impl std::fmt::Display for ManifestParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}
impl std::error::Error for ManifestParseError {}

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

impl Default for Manifest {
  fn default() -> Self {
    Self {
      difftool: Difftool::Git,
      since: "1 week ago".to_string(),
      since_datetime: None,
      remotes: HashMap::new(),
      root: None,
    }
  }
}

impl Manifest {
  pub fn new(path: std::path::PathBuf, root: PathBuf) -> Result<Manifest, Box<dyn std::error::Error>> {
    let manifest_contents =
      std::fs::read_to_string(&path).with_context(|| format!("Error reading file: `{}`", &path.to_str().unwrap()))?;

    let mut manifest: Manifest = toml::from_str(&manifest_contents)?;
    let since_datetime = time::parse_relative(&manifest.since, &chrono::Utc::now())?;
    if let Difftool::Fallthrough(difftool) = &manifest.difftool
      && difftool.is_empty()
    {
      return Err(Box::new(ManifestParseError(
        "When difftool is present in manifest, it must have length > 0".to_string(),
      )));
    }
    manifest.root = Some(root);
    manifest.since_datetime = Some(since_datetime);
    Ok(manifest)
  }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Remote {
  pub(crate) name: String,
  pub(crate) origin: String,
  pub(crate) branch: Option<String>,
}

fn default_difftool() -> Difftool {
  Difftool::Git
}
