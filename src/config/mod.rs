use crate::app::AppResult;
use crate::cli::ManifestCliArgs;
use crate::difftool::Difftool;
use crate::manifest::{self, Manifest};
use crate::theme::Theme;
use crate::time;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Settings for a diff, from a manifest overridden by the global config file
/// at `~/.config/dyd/dyd.toml`, then by CLI args.
#[derive(Debug)]
pub struct Config {
  pub difftool: Difftool,
  pub remotes: HashMap<String, Remote>,
  pub root: PathBuf,
  pub since: chrono::DateTime<chrono::Utc>,
  pub theme: Theme,
}

impl TryFrom<(ManifestCliArgs, Manifest, PathBuf)> for Config {
  type Error = Box<dyn std::error::Error>;

  fn try_from((args, manifest, cache_path): (ManifestCliArgs, Manifest, PathBuf)) -> AppResult<Self> {
    let mut config = Self {
      difftool: manifest.difftool,
      remotes: manifest
        .remotes
        .into_iter()
        .map(|(id, remote)| (id, remote.into()))
        .collect(),
      root: cache_path,
      since: time::parse_relative(&manifest.since, &chrono::Utc::now())?,
      theme: Theme::default(),
    };
    Overrides::load(&global_path()?)?.apply(&mut config);
    if let Some(theme) = args.theme {
      config.theme = theme;
    }
    Ok(config)
  }
}

#[derive(Debug)]
pub struct Remote {
  pub branch: Option<String>,
  pub name: String,
  pub origin: String,
}

impl From<manifest::Remote> for Remote {
  fn from(remote: manifest::Remote) -> Self {
    Self {
      branch: remote.branch,
      name: remote.name,
      origin: remote.origin,
    }
  }
}

/// Values in the global config file.
#[derive(Debug, Default, Deserialize)]
struct Overrides {
  difftool: Option<Difftool>,
  theme: Option<Theme>,
}

impl Overrides {
  fn load(path: &Path) -> Result<Self> {
    if !path.exists() {
      return Ok(Self::default());
    }

    let contents = fs::read_to_string(path)?;
    Self::parse(&contents)
  }

  fn parse(contents: &str) -> Result<Self> {
    Ok(toml::from_str(contents)?)
  }

  fn apply(self, config: &mut Config) {
    if let Some(difftool) = self.difftool {
      config.difftool = difftool;
    }
    if let Some(theme) = self.theme {
      config.theme = theme;
    }
  }
}

fn global_path() -> Result<PathBuf> {
  let home = std::env::var("HOME")?;
  Ok(PathBuf::from(home).join(".config/dyd/dyd.toml"))
}

#[cfg(test)]
mod tests {
  use super::{Config, Overrides};
  use crate::difftool::Difftool;
  use crate::theme::Theme;

  fn config() -> Config {
    Config {
      difftool: Difftool::Git,
      remotes: Default::default(),
      root: "/nonexistent".into(),
      since: chrono::Utc::now(),
      theme: Theme::Auto,
    }
  }

  #[test]
  fn empty_overrides_keep_values() {
    let mut config = config();
    Overrides::parse("").unwrap().apply(&mut config);
    assert!(matches!(config.difftool, Difftool::Git));
    assert!(matches!(config.theme, Theme::Auto));
  }

  #[test]
  fn overrides_replace_values() {
    let mut config = config();
    Overrides::parse("difftool = \"ksdiff ${DIFF}\"\ntheme = \"dark\"")
      .unwrap()
      .apply(&mut config);
    assert!(matches!(config.difftool, Difftool::Fallthrough(d) if d == "ksdiff ${DIFF}"));
    assert!(matches!(config.theme, Theme::Dark));
  }

  #[test]
  fn overrides_reject_empty_difftool() {
    assert!(Overrides::parse("difftool = \"\"").is_err());
  }
}
