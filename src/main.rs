use dyd::manifest::Manifest;

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "dyd.toml")]
    manifest: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let manifest_contents = std::fs::read_to_string(&args.manifest)
        .with_context(|| format!("Error reading file: `{}`", &args.manifest.to_str().unwrap()))?;

    let manifest: Manifest = toml::from_str(&manifest_contents)?;

    println!("manifest: {:?}", manifest);
    Ok(())
}
