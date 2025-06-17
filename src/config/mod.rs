use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
pub struct Config {}

impl Config {
  pub fn load() -> Result<Self> {
    let config_path = config_path()?;

    if !config_path.exists() {
      return Ok(Self::default());
    }

    let contents = fs::read_to_string(&config_path)?;
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
  }
}

fn config_path() -> Result<PathBuf> {
  let home = std::env::var("HOME")?;
  Ok(PathBuf::from(home).join(".config/dyd/dyd.toml"))
}
