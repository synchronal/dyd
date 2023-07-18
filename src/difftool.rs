use serde::de::{value, Deserializer, IntoDeserializer};
use serde::Deserialize;
use std::str::FromStr;

use crate::repo::Repo;

#[derive(Debug)]
pub enum Difftool {
  Git,
  GitHub,
  Fallthrough(String),
}

impl Default for Difftool {
  fn default() -> Self {
    Self::Git
  }
}

impl Difftool {
  pub fn to_str(&self, _repo: &Repo) -> String {
    match self {
      Difftool::Git => "git difftool -g -y ${DIFF}".to_owned(),
      Difftool::GitHub => todo!(),
      Difftool::Fallthrough(difftool) => difftool.clone(),
    }
  }
}

impl std::fmt::Display for Difftool {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Difftool::Git => write!(f, "git"),
      Difftool::GitHub => write!(f, "github"),
      Difftool::Fallthrough(difftool) => write!(f, "fallthrough: {difftool}"),
    }
  }
}

impl FromStr for Difftool {
  type Err = value::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::deserialize(s.into_deserializer())
  }
}

impl<'de> Deserialize<'de> for Difftool {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;

    let deserialized = if s == "git".to_owned() {
      Self::Git
    } else if s == "github".to_owned() {
      Self::GitHub
    } else {
      Self::Fallthrough(s)
    };
    Ok(deserialized)
  }
}
