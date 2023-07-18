use crate::repo::{Log, Repo};
use regex::Regex;
use serde::de::{value, Deserializer, IntoDeserializer};
use serde::Deserialize;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

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
  pub fn open(&self, root_path: &Path, repo: &Repo, log: &Log) {
    let mut cmd: String = "".to_string();
    let mut args: Vec<String> = vec![];
    let ref_to: String = match repo.branch.clone() {
      Some(branch) => format!("origin/{branch}"),
      None => "HEAD".into(),
    };
    let diff = format!("{}..{ref_to}", log.sha);
    let repo_path = repo.path(root_path).unwrap();

    let cwd = std::env::current_dir()
      .unwrap()
      .into_os_string()
      .into_string()
      .unwrap();

    let mut context = std::collections::HashMap::new();
    context.insert("DYD_PWD".to_string(), cwd.clone());
    context.insert("DIFF".to_string(), diff.clone());
    context.insert("ORIGIN".to_string(), repo.origin.clone());
    context.insert("REF_FROM".to_string(), log.sha.clone());
    context.insert("REF_TO".to_string(), ref_to.clone());
    assert!(envsubst::validate_vars(&context).is_ok());

    let difftool_expansion = envsubst::substitute(self.to_str(repo, &log.sha, &repo.branch), &context).unwrap();

    let difftool_parts: Vec<&str> = difftool_expansion.split(' ').collect();
    difftool_parts
      .iter()
      .enumerate()
      .for_each(|(index, value)| {
        if index == 0 {
          cmd = value.to_string();
        } else {
          args.push(value.to_string());
        }
      });

    match Command::new(cmd)
      .args(args)
      .env("DYD_PWD", cwd)
      .env("DIFF", diff)
      .env("REF_FROM", &log.sha)
      .env("REF_TO", ref_to)
      .env("ORIGIN", &repo.origin)
      .current_dir(repo_path)
      .output()
    {
      Ok(_) => (),
      Err(err) => eprintln!("\rError opening difftool:\r\n{err:?}\r\ndifftool: {self}"),
    };
  }

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
