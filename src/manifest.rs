use crate::time;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
struct ManifestParseError(String);

impl std::fmt::Display for ManifestParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "There is an error: {}", self.0)
  }
}
impl std::error::Error for ManifestParseError {}

#[derive(Debug, Deserialize)]
pub enum TimeZoneCast {
  #[serde(rename = "as-is")]
  AsIs,
  #[serde(rename = "local")]
  Local,
  #[serde(rename = "utc")]
  Utc,
}

impl Default for TimeZoneCast {
  fn default() -> Self {
    TimeZoneCast::AsIs
  }
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
  #[serde(default = "default_difftool")]
  pub(crate) difftool: String,
  pub(crate) since: String,
  #[serde(default)]
  pub(crate) timezones: TimeZoneCast,
  #[serde(skip)]
  pub(crate) since_datetime: Option<chrono::DateTime<chrono::Utc>>,
  pub(crate) remotes: HashMap<String, Remote>,
  pub(crate) root: Option<PathBuf>,
}

impl Default for Manifest {
  fn default() -> Self {
    Self {
      difftool: "git difftool -g -y ${DIFF}".to_string(),
      remotes: HashMap::new(),
      root: None,
      since: "1 week ago".to_string(),
      since_datetime: None,
      timezones: TimeZoneCast::AsIs,
    }
  }
}

impl Manifest {
  pub fn new(path: std::path::PathBuf, root: PathBuf) -> Result<Manifest, Box<dyn std::error::Error>> {
    let manifest_contents =
      std::fs::read_to_string(&path).with_context(|| format!("Error reading file: `{}`", &path.to_str().unwrap()))?;

    let mut manifest: Manifest = toml::from_str(&manifest_contents)?;
    let since_datetime = time::parse_relative(&manifest.since, &chrono::Utc::now())?;
    if manifest.difftool.is_empty() {
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
}

fn default_difftool() -> String {
  "git difftool -g -y ${DIFF}".to_string()
}
