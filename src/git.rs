use crate::repo::{Log, Repo};

use std::path::PathBuf;
use std::process::Command;

static GIT_FORMAT: &str = "%h\x0B%ct\x0B%ch\x0B%an\x0B%s";

pub fn clone(origin: &String, path: &PathBuf) {
    let path_str = path.clone();

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
            "100",
            "--abbrev-commit",
            "--color=always",
            &format!("--pretty=tformat:{}", GIT_FORMAT),
        ])
        .current_dir(path)
        .output()
        .expect("failed to retrieve git log")
        .stdout
}

pub fn open_difftool(root_path: &PathBuf, repo: &Repo, log: &Log) {
    let diff = format!("{}..head", log.sha);
    let repo_path = repo.path(root_path).unwrap();

    Command::new("git")
        .args(["difftool", "-g", &diff])
        .current_dir(repo_path)
        .output()
        .unwrap();
}

pub fn pull(path: &PathBuf) {
    Command::new("git")
        .args(["pull"])
        .current_dir(path)
        .output()
        .unwrap();
}
