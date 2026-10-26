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
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io::Write;
use std::path::PathBuf;

pub mod app;
pub mod cli;
pub mod config;
pub mod difftool;
pub mod git;
pub mod manifest;
pub mod refresh;
pub mod semaphore;
pub mod terminal;
pub mod theme;
pub mod time;
pub mod ui;
pub mod widget;

pub fn clean(cache_path: PathBuf, verbose: bool) -> AppResult<()> {
  if cache_path.exists() {
    if verbose {
      for repo in cache_path
        .read_dir()
        .expect("Expected .cache/dyd to be a directory")
        .flatten()
      {
        println!("clean: {:?}", repo.path());
        log::info!("clean: {:?}", repo.path());
        if repo.file_type()?.is_dir() {
          std::fs::remove_dir_all(repo.path())?;
        } else {
          std::fs::remove_file(repo.path())?;
        }
      }
    }
    log::info!("clean: {cache_path:?}");
    std::fs::remove_dir_all(cache_path)?;
  }
  Ok(())
}

pub fn diff(manifest: PathBuf, cache_path: PathBuf, theme: ColorTheme) -> AppResult<()> {
  let manifest = Manifest::new(manifest, cache_path)?;
  let mut app: App = App::from_manifest(manifest, theme);
  app.request_refresh();

  let backend = CrosstermBackend::new(std::io::stderr());
  let terminal = Terminal::new(backend)?;
  let events = EventHandler::new(250);
  let mut tui = Tui::new(terminal, events);

  crate::terminal::tui::install_panic_hook();
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
      Event::RepoUpdateDone => app.finish_repo_update(),
    }
  }

  tui.exit()?;

  Ok(())
}

pub fn write_default_manifest(manifest_path: PathBuf) -> AppResult<()> {
  println!("Creating file: {:?}", manifest_path);

  if let Some(parent) = manifest_path.parent() {
    std::fs::create_dir_all(parent)?;
  }

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
  writeln!(file, "origin = \"git@github.com:synchronal/dyd\"")?;
  writeln!(file)?;
  Ok(())
}

#[cfg(test)]
mod tests {
  #[test]
  fn write_default_manifest_creates_parent_directory() -> super::AppResult<()> {
    let dir = std::env::temp_dir().join(format!("dyd-init-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest_path = dir.join(".config/dyd.toml");

    super::write_default_manifest(manifest_path.clone())?;

    assert!(manifest_path.exists());
    std::fs::remove_dir_all(&dir)?;
    Ok(())
  }

  #[test]
  fn clean_verbose_removes_files_and_directories() -> super::AppResult<()> {
    let cache_path = std::env::temp_dir().join(format!("dyd-clean-test-{}", std::process::id()));
    std::fs::create_dir_all(cache_path.join("github.com/synchronal/dyd"))?;
    std::fs::write(cache_path.join(".DS_Store"), "")?;

    super::clean(cache_path.clone(), true)?;

    assert!(!cache_path.exists());
    Ok(())
  }
}
