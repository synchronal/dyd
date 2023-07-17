use crate::app::AppResult;
use crate::difftool::Difftool;
use crate::repo::{Log, Repo};

use std::path::{Path, PathBuf};
use std::process::Command;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone_repo(origin: &String, path: &Path) {
  let path_str = path.to_path_buf();

  Command::new("git")
    .args(["clone", origin, path_str.to_str().unwrap()])
    .output()
    .unwrap();
}

pub fn logs(path: &PathBuf, branch: &Option<String>) -> AppResult<Vec<u8>> {
  let mut logs = Command::new("git");
  logs
    .args([
      "log",
      "--date=local",
      "-n",
      "400",
      "--abbrev-commit",
      "--color=always",
      &format!("--pretty=tformat:{GIT_FORMAT}"),
    ])
    .current_dir(path);

  if let Some(branch) = branch {
    logs.arg(format!("origin/{branch}"));
  }

  Ok(logs.output()?.stdout)
}

pub fn open_difftool(root_path: &Path, difftool: &Difftool, repo: &Repo, log: &Log) {
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
  let difftool_expansion = envsubst::substitute(format!("{difftool}"), &context).unwrap();

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
    Err(err) => eprintln!("\rError opening difftool:\r\n{err:?}\r\ndifftool: {difftool}"),
  };
}

pub fn pull_repo(path: &PathBuf) {
  Command::new("git")
    .args(["pull", "--prune"])
    .current_dir(path)
    .output()
    .unwrap();
}

pub fn switch_branch(path: &PathBuf, branch: String) {
  Command::new("git")
    .args(["switch", &branch])
    .current_dir(path)
    .output()
    .unwrap();
}
