use crate::repo::{Log, Repo};

use std::path::{Path, PathBuf};
use std::process::Command;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone(origin: &String, path: &Path) {
    let path_str = path.to_path_buf();

    Command::new("git")
        .args(["clone", origin, path_str.to_str().unwrap()])
        .output()
        .unwrap();
}

pub fn logs(path: &PathBuf) -> Vec<u8> {
    Command::new("git")
        .args([
            "log",
            "--date=local",
            "-n",
            "400",
            "--abbrev-commit",
            "--color=always",
            &format!("--pretty=tformat:{}", GIT_FORMAT),
        ])
        .current_dir(path)
        .output()
        .expect("failed to retrieve git log")
        .stdout
}

pub fn open_difftool(root_path: &Path, difftool: &String, repo: &Repo, log: &Log) {
    let mut cmd: String = "".to_string();
    let mut args: Vec<String> = vec![];
    let diff = format!("{}..HEAD", log.sha);
    let repo_path = repo.path(root_path).unwrap();

    let mut context = std::collections::HashMap::new();
    context.insert("DIFF".to_string(), diff.clone());
    context.insert("ORIGIN".to_string(), repo.origin.clone());
    context.insert("REF_FROM".to_string(), log.sha.clone());
    context.insert("REF_TO".to_string(), "HEAD".to_string());
    assert!(envsubst::validate_vars(&context).is_ok());
    let difftool_expansion = envsubst::substitute(difftool, &context).unwrap();

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
        .env("DIFF", diff)
        .env("REF_FROM", &log.sha)
        .env("REF_TO", "HEAD")
        .env("ORIGIN", &repo.origin)
        .current_dir(repo_path)
        .output()
    {
        Ok(_) => (),
        Err(err) => eprintln!("\rError opening difftool:\r\n{:?}\r\ndifftool: {}", err, difftool),
    };
}

pub fn pull(path: &PathBuf) {
    Command::new("git")
        .args(["pull"])
        .current_dir(path)
        .output()
        .unwrap();
}
