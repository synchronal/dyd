use crate::theme;
use clap::Args;
use clap::Parser;
use clap::Subcommand;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(version)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct CLI {
  #[command(subcommand)]
  pub command: Option<Command>,
  #[clap(flatten)]
  pub diff: ManifestCliArgs,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Remove all repositories that have been checked out to the local cache.
  Clean {
    /// verbose
    #[arg(short, long, action)]
    verbose: bool,
  },
  /// Open the daily diff. Defaults to (-m ./.config/dyd.toml), then (-m ./dyd.toml).
  Diff(ManifestCliArgs),
  /// Generate a (toml-encoded) manifest for defining repos to diff. Defaults to
  /// (-m ./.config/dyd.toml).
  Init(ManifestCliArgs),
}

#[derive(Args, Debug)]
pub struct ManifestCliArgs {
  /// path to a toml-formatted manifest file. Defaults to ./.config/dyd.toml if
  /// present, otherwise ./dyd.toml.
  #[clap(value_parser)]
  #[arg(short, long, env = "DYD_MANIFEST_PATH", value_hint = clap::ValueHint::FilePath)]
  pub manifest: Option<PathBuf>,

  /// Color theme
  #[arg(short, long, env = "DYD_THEME", value_hint = clap::ValueHint::FilePath)]
  pub theme: Option<theme::Theme>,
}

impl CLI {
  #[allow(clippy::new_without_default)]
  pub fn new() -> Self {
    CLI::parse()
  }
}

impl ManifestCliArgs {
  pub fn manifest_path(&self) -> PathBuf {
    self
      .manifest
      .clone()
      .unwrap_or_else(|| default_manifest_path(Path::new(".")))
  }

  pub fn init_manifest_path(&self) -> PathBuf {
    self
      .manifest
      .clone()
      .unwrap_or_else(|| PathBuf::from(".config/dyd.toml"))
  }
}

fn default_manifest_path(dir: &Path) -> PathBuf {
  let config_manifest = dir.join(".config/dyd.toml");
  if config_manifest.exists() {
    config_manifest
  } else {
    dir.join("dyd.toml")
  }
}

#[cfg(test)]
mod tests {
  use super::default_manifest_path;

  fn temp_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dyd-cli-test-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
  }

  #[test]
  fn default_manifest_path_prefers_config_dir() {
    let dir = temp_dir("config");
    std::fs::create_dir_all(dir.join(".config")).unwrap();
    std::fs::write(dir.join(".config/dyd.toml"), "").unwrap();
    std::fs::write(dir.join("dyd.toml"), "").unwrap();

    assert_eq!(default_manifest_path(&dir), dir.join(".config/dyd.toml"));
    std::fs::remove_dir_all(&dir).unwrap();
  }

  #[test]
  fn default_manifest_path_falls_back_to_dyd_toml() {
    let dir = temp_dir("fallback");

    assert_eq!(default_manifest_path(&dir), dir.join("dyd.toml"));
    std::fs::remove_dir_all(&dir).unwrap();
  }
}
