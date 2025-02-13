use dyd::app::handler::handle_key_events;
use dyd::app::{App, AppResult};
use dyd::app::{Event, EventHandler};
use dyd::cli::{Command, CLI};
use dyd::manifest::Manifest;
use dyd::terminal::Tui;

use anyhow::Context;

use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::encode::pattern::PatternEncoder;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() -> AppResult<()> {
  let cli = CLI::new();
  let command = cli.command.unwrap_or(Command::Diff(cli.diff));

  let share_path = setup_dyd_share_path()?;
  let state_path = setup_dyd_state_path()?;
  setup_logger(state_path)?;

  match command {
    Command::Clean { verbose } => clean(verbose),
    Command::Diff(args) => diff(args.manifest, share_path),
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
        println!("clean: {:?}", repo.path());
        log::info!("clean: {:?}", repo.path());
        std::fs::remove_dir_all(repo.path())?;
      }
    }
    log::info!("clean: {:?}", path);
    std::fs::remove_dir_all(path)?;
  }
  Ok(())
}

fn diff(manifest: PathBuf, share_path: PathBuf) -> AppResult<()> {
  let manifest = Manifest::new(manifest, share_path)?;
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

fn write_default_manifest(manifest_path: PathBuf) -> AppResult<()> {
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

fn setup_dyd_share_path() -> AppResult<PathBuf> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".local/share/dyd");
  std::fs::create_dir_all(&path)?;
  Ok(path)
}

fn setup_dyd_state_path() -> AppResult<PathBuf> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".local/state/dyd");
  std::fs::create_dir_all(&path)?;
  Ok(path)
}

fn setup_logger(state_path: PathBuf) -> AppResult<()> {
  let trigger_size: u64 = 1000000; // 1MB
  let trigger = Box::new(SizeTrigger::new(trigger_size));
  let roller = Box::new(DeleteRoller::new());

  let compound_policy = Box::new(CompoundPolicy::new(trigger, roller));

  let appender = RollingFileAppender::builder()
    .encoder(Box::new(PatternEncoder::new("[{d}][{h({l})}] {m}{n}")))
    .build(Path::new(&state_path).join("dyd.log"), compound_policy)?;

  let log_level = std::env::var("DEBUG").map_or(log::LevelFilter::Info, |_| log::LevelFilter::Debug);

  let config = log4rs::Config::builder()
    .appender(log4rs::config::Appender::builder().build("file", Box::new(appender)))
    .build(
      log4rs::config::Root::builder()
        .appender("file")
        .build(log_level),
    )?;

  let _handle = log4rs::init_config(config).unwrap();

  Ok(())
}
