use dyd::app::AppResult;
use dyd::cli::{CLI, Command};
use dyd::config::Config;
use dyd::manifest::Manifest;

use anyhow::Context;

use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::encode::pattern::PatternEncoder;
use std::path::{Path, PathBuf};

fn main() -> AppResult<()> {
  let cli = CLI::new();
  let command = cli.command.unwrap_or(Command::Diff(cli.diff));

  let _ = setup_dyd_config_path()?;
  let cache_path = setup_dyd_cache_path()?;
  let state_path = setup_dyd_state_path()?;
  setup_logger(state_path)?;

  match command {
    Command::Clean { verbose } => dyd::clean(cache_path, verbose),
    Command::Diff(args) => {
      let manifest = Manifest::new(args.manifest_path(), cache_path)?;
      dyd::diff(Config::try_from((args, manifest))?)
    }
    Command::Init(args) => dyd::write_default_manifest(args.init_manifest_path()),
  }
}

fn setup_dyd_config_path() -> AppResult<PathBuf> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".config/dyd");
  std::fs::create_dir_all(&path)?;
  Ok(path)
}

fn setup_dyd_cache_path() -> AppResult<PathBuf> {
  let home = std::env::var("HOME").context("Unable to access HOME")?;
  let path = Path::new(&home).join(".cache/dyd");
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
