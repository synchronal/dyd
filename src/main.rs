use dyd::app::{App, AppResult};
use dyd::cli::{CLI, Command};
use dyd::event::{Event, EventHandler};
use dyd::handler::handle_key_events;
use dyd::manifest::Manifest;
use dyd::tui::Tui;

use anyhow::Context;
use std::path::{Path, PathBuf};
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
    let args = CLI::new();

    match args.command {
        Some(Command::Diff{manifest}) => {
            diff(manifest)
        }
        None => {
            diff(std::path::PathBuf::from("dyd.toml"))
        }
    }
}

fn diff(manifest: std::path::PathBuf) -> AppResult<()> {
    let path = setup_dyd_path()?;
    let manifest = Manifest::new(manifest, path)?;
    let mut app: App = manifest.into();

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;

        match tui.events.next()? {
            Event::Tick(sender) => app.tick(sender)?,
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::RepoStatusChange(id, state) => app.update_repo_status(id, state)?,
            Event::RepoStatusComplete(id, logs) => app.update_repo_logs(id, logs)?,
        }
    }

    tui.exit()?;

    Ok(())
}

fn setup_dyd_path() -> AppResult<PathBuf> {
    let home = std::env::var("HOME").context("Unable to access HOME")?;
    let path = Path::new(&home).join(".local/share/dyd");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}
