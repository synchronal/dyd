#![cfg_attr(feature = "strict", deny(warnings))]

//! DYD is a CLI for diffing multiple git repositories.
//!
//! It relies on the shell for authentication to remote origins, and uses
//! the configured GUI git difftool for opening diffs.

use crate::app::handler::handle_key_events;
use crate::app::{App, AppResult, Event, EventHandler};
use crate::manifest::Manifest;
use crate::terminal::Tui;
use crate::theme::ColorTheme;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Write;
use std::path::PathBuf;

pub mod app;
pub mod cli;
pub mod config;
pub mod difftool;
pub mod git;
pub mod manifest;
pub mod semaphore;
pub mod terminal;
pub mod theme;
pub mod time;
pub mod ui;
pub mod widget;

pub fn clean(share_path: PathBuf, verbose: bool) -> AppResult<()> {
  if share_path.exists() {
    if verbose {
      for repo in share_path
        .read_dir()
        .expect("Expected .local/share/dyd to be a directory")
        .flatten()
      {
        println!("clean: {:?}", repo.path());
        log::info!("clean: {:?}", repo.path());
        std::fs::remove_dir_all(repo.path())?;
      }
    }
    log::info!("clean: {share_path:?}");
    std::fs::remove_dir_all(share_path)?;
  }
  Ok(())
}

pub fn diff(manifest: PathBuf, share_path: PathBuf, theme: ColorTheme) -> AppResult<()> {
  let manifest = Manifest::new(manifest, share_path)?;
  let mut app: App = App::from_manifest(manifest, theme);

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
      Event::RepoStatusChange(id, state) => app.update_repo_status(&id, state)?,
      Event::RepoStatusComplete(id, logs) => app.update_repo_logs(&id, logs)?,
    }
  }

  tui.exit()?;

  Ok(())
}

pub fn write_default_manifest(manifest_path: PathBuf) -> AppResult<()> {
  println!("Creating file: {:?}", &manifest_path);

  let mut file = std::fs::OpenOptions::new()
    .create_new(true)
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
