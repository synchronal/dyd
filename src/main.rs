use dyd::app::{App, AppResult};
use dyd::cli::{Command, CLI};
use dyd::app::{Event, EventHandler};
use dyd::app::handler::handle_key_events;
use dyd::manifest::Manifest;
use dyd::terminal::Tui;

use anyhow::Context;
use std::io::Write;
use std::path::{Path, PathBuf};
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> AppResult<()> {
  let cli = CLI::new();
  let command = cli.command.unwrap_or(Command::Diff(cli.diff));

  match command {
    Command::Clean { verbose } => clean(verbose),
    Command::Diff(args) => diff(args.manifest),
    Command::Init(args) => write_default_manifest(args.manifest),
  }
}

fn clean(verbose: bool) -> AppResult<()> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".local/share/dyd");
  if path.exists() {
    if verbose {
      for repo in path
        .read_dir()
        .expect("Expected .local/share/dyd to be a directory")
        .flatten()
      {
        println!("Removing {:?}", repo.path());
        std::fs::remove_dir_all(repo.path())?;
      }
    }
    std::fs::remove_dir_all(path)?;
  }
  Ok(())
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

fn write_default_manifest(manifest_path: std::path::PathBuf) -> AppResult<()> {
  println!("Creating file: {:?}", &manifest_path);

  let mut file = std::fs::OpenOptions::new()
    .create_new(true)
    .write(true)
    .append(true)
    .open(manifest_path)?;

  writeln!(file, "since = \"5 days ago\"")?;
  writeln!(file)?;
  writeln!(file, "[remotes]")?;
  writeln!(file)?;
  writeln!(file, "[remotes.dyd]")?;
  writeln!(file, "name = \"Daily diff\"")?;
  writeln!(file, "origin = \"git@github.com:sychronal/dyd\"")?;
  writeln!(file)?;
  Ok(())
}

fn setup_dyd_path() -> AppResult<PathBuf> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".local/share/dyd");
  std::fs::create_dir_all(&path)?;
  Ok(path)
}
