use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
pub struct CLI {
  #[command(subcommand)]
  pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
  /// Open the daily diff. Default to (-m ./dyd.toml)
  Diff {
    /// path to a toml
    #[arg(short, long, default_value = "dyd.toml")]
    manifest: std::path::PathBuf,
  },
  /// Generate a (toml-encoded) manifest for defining repos to diff.
  Init {
    /// path to a toml
    #[arg(short, long, default_value = "dyd.toml")]
    manifest: std::path::PathBuf,
  },
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
