use clap::Parser;

#[derive(Parser, Debug)]
pub struct CLI {
    #[clap(short, long, default_value = "dyd.toml")]
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
