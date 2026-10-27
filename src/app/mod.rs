pub mod event;
pub mod handler;

pub use self::event::{Event, EventHandler};
use crate::config::Config;
use crate::difftool::Difftool;
use crate::git::repo::{Log, Repo, RepoStatus};
use crate::refresh::Refresh;
use crate::semaphore::Semaphore;
use crate::theme::ColorTheme;
use crate::ui;
use chrono::Local;
use indexmap::map::IndexMap;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::TableState;
use std::cmp::Ordering;
use std::error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
  #[default]
  Idle,
  RefreshRequested,
}

/// Selected pane
#[derive(Debug, Default, PartialEq, Eq)]
pub enum SelectedPane {
  Diff,
  #[default]
  Repos,
}

/// Modal
#[derive(Debug, Default, PartialEq, Eq)]
pub enum SelectedModal {
  #[default]
  None,
  Calendar,
}

/// Maximum number of concurrent git operations.
const MAX_CONCURRENT_GIT_OPS: usize = 5;

#[derive(Debug)]
pub struct App {
  pub calendar_state: crate::widget::calendar::CalendarState,
  pub difftool: Difftool,
  pub modal: SelectedModal,
  pub repo_state: TableState,
  pub repos: IndexMap<String, Repo>,
  pub root_path: PathBuf,
  pub running: bool,
  pub selected_pane: SelectedPane,
  pub selected_repo_state: TableState,
  pub since: chrono::DateTime<chrono::Utc>,
  pub state: AppState,
  pub theme: ColorTheme,
  pub timezone_offset: chrono::offset::FixedOffset,
  refresh: Refresh,
  semaphore: Arc<Semaphore>,
}

impl App {
  pub fn from_config(config: Config) -> AppResult<Self> {
    let repos: IndexMap<String, Repo> = config
      .remotes
      .into_iter()
      .map(|(id, remote)| (id, remote.into()))
      .collect();

    let mut repo_state = TableState::default();
    if !repos.is_empty() {
      repo_state.select(Some(0))
    }

    let mut selected_repo_state = TableState::default();
    selected_repo_state.select(Some(0));

    let since = config.since;
    let calendar_state = (&since).into();

    let offset_sec = Local::now().offset().local_minus_utc();
    let offset = chrono::offset::FixedOffset::east_opt(offset_sec).unwrap();

    Ok(Self {
      calendar_state,
      difftool: config.difftool,
      modal: SelectedModal::default(),
      repo_state,
      repos,
      root_path: config.root,
      refresh: Refresh::default(),
      running: true,
      selected_pane: SelectedPane::default(),
      selected_repo_state,
      semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_GIT_OPS)),
      since,
      state: AppState::default(),
      theme: config.theme.try_into()?,
      timezone_offset: offset,
    })
  }
  pub fn tick(&mut self, sender: mpsc::Sender<Event>) -> AppResult<()> {
    if self.state == AppState::RefreshRequested {
      self.update(sender)?;
      self.state = AppState::Idle;
    }
    Ok(())
  }

  pub fn render(&mut self, frame: &mut Frame) {
    let size = frame.area();
    let layout = Layout::default()
      .direction(Direction::Horizontal)
      .margin(0)
      .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
      .split(size);

    let sidebar = Layout::default()
      .direction(Direction::Vertical)
      .margin(0)
      .constraints([Constraint::Ratio(7, 10), Constraint::Ratio(3, 10)].as_ref())
      .split(layout[1]);

    frame.render_stateful_widget(ui::diff::render(self), layout[0], &mut self.selected_repo_state.clone());
    frame.render_stateful_widget(ui::repos::render(self), sidebar[0], &mut self.repo_state.clone());
    frame.render_widget(ui::help::render(self), sidebar[1]);

    ui::modal::render(self, frame);
  }

  /// Pulls or clones every repo on the next tick, unless a refresh is already
  /// requested or running.
  pub fn request_refresh(&mut self) {
    if self.state == AppState::Idle && !self.refresh.is_running() {
      self.state = AppState::RefreshRequested;
    }
  }

  pub fn update(&mut self, sender: mpsc::Sender<Event>) -> AppResult<()> {
    for (id, repo) in &self.repos {
      repo.update(
        id,
        &self.root_path,
        sender.clone(),
        Arc::clone(&self.semaphore),
        self.refresh.track(sender.clone()),
      )?;
    }
    Ok(())
  }

  pub fn finish_repo_update(&mut self) {
    self.refresh.finish();
  }

  pub fn update_repo_status(&mut self, id: &str, status: RepoStatus) -> AppResult<()> {
    if let Some(repo) = self.repos.get_mut(id) {
      repo.status = status;
    }
    Ok(())
  }

  pub fn update_repo_logs(&mut self, id: &str, logs: Vec<Log>) -> AppResult<()> {
    if let Some(repo) = self.repos.get_mut(id) {
      repo.logs = logs;
      repo.status = RepoStatus::Finished;
    }

    self.repos.sort_unstable_by(&Self::sort_repos);

    Ok(())
  }

  #[allow(clippy::ptr_arg)]
  fn sort_repos(_key1: &String, repo1: &Repo, _key2: &String, repo2: &Repo) -> Ordering {
    repo1.cmp(repo2)
  }
}
