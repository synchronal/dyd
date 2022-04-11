use dyd::manifest::Manifest;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::thread;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders};
use tui::Terminal;

#[derive(Parser)]
struct Cli {
    #[clap(short, long, default_value = "dyd.toml")]
    manifest: std::path::PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _manifest = parse_manifest();

    enable_raw_mode()?;

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default().title("Diff").borders(Borders::ALL);
        f.render_widget(block, size);
    })?;

    thread::sleep(Duration::from_millis(5000));

    disable_raw_mode()?;

    Ok(())
}

fn parse_manifest() -> Result<Manifest, Box<dyn std::error::Error>> {
let args = Cli::parse();
    let manifest_contents = std::fs::read_to_string(&args.manifest)
        .with_context(|| format!("Error reading file: `{}`", &args.manifest.to_str().unwrap()))?;

    let manifest: Manifest = toml::from_str(&manifest_contents)?;
    Ok(manifest)
}
