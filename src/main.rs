use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "dyd.toml")]
    manifest: std::path::PathBuf,
}

fn main() {
    let _args = Cli::parse();
}
