use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Diff {
        #[clap(short, long, default_value = "dyd.toml")]
        manifest: std::path::PathBuf,
    },
    Init {
        #[clap(short, long, default_value = "dyd.toml")]
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
