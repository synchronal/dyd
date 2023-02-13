use clap::Args;
// use clap::Arg;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct CLI {
  #[command(subcommand)]
  pub command: Option<Command>,
  #[command(flatten)]
  pub diff: Option<ManifestCliArgs>,
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
  #[arg(short, long, env = "DYD_MANIFEST_PATH", default_value = "dyd.toml")]
  pub manifest: std::path::PathBuf,
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
