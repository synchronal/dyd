use crate::theme;
use clap::Args;
use clap::Parser;
use clap::Subcommand;

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
  /// Open the daily diff. Defaults to (-m ./dyd.toml).
  Diff(ManifestCliArgs),
  /// Generate a (toml-encoded) manifest for defining repos to diff.
  Init(ManifestCliArgs),
}

#[derive(Args, Debug)]
pub struct ManifestCliArgs {
  /// path to a toml-formatted manifest file.
  #[clap(value_parser)]
  #[arg(short, long, env = "DYD_MANIFEST_PATH", default_value = "dyd.toml", value_hint = clap::ValueHint::FilePath)]
  pub manifest: std::path::PathBuf,

  /// Color theme
  #[arg(short, long, env = "DYD_THEME", default_value = "auto", value_hint = clap::ValueHint::FilePath)]
  pub theme: theme::Theme,
}

impl Default for CLI {
  fn default() -> Self {
    Self::new()
  }
}

impl CLI {
  pub fn new() -> Self {
    CLI::parse()
  }
}
