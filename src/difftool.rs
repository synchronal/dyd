use regex::Regex;
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
  pub fn to_str(&self, repo: &Repo, from_sha: &String, branch: &Option<String>) -> String {
    match self {
      Difftool::Git => "git difftool -g -y ${DIFF}".to_owned(),
      Difftool::GitHub => Difftool::open_github(repo, from_sha, branch),
      Difftool::Fallthrough(difftool) => difftool.clone(),
    }
  }

  fn open_github(repo: &Repo, from_sha: &String, branch: &Option<String>) -> String {
    let mut github_url = repo.origin.clone();
    github_url = github_url
      .trim()
      .replace(':', "/")
      .replace("git@", "https://");
    let re = Regex::new(r"\.git$").unwrap();
    let origin = re.replace_all(&github_url, "");
    let ref_to = branch.clone().unwrap_or("HEAD".to_owned());
    format!("open {origin}/compare/{from_sha}..{ref_to}?diff=split")
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
