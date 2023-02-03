use crate::event::Event;
use crate::manifest::{self, Manifest};
use crate::repo::{Log, Repo, RepoStatus};
use crate::ui;
use crate::widget::calendar::CalendarState;

use indexmap::map::IndexMap;
use std::cmp::Ordering;
use std::error;
use std::path::PathBuf;
use std::sync::mpsc;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::terminal::Frame;
use tui::widgets::TableState;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
  #[default]
  Init,
  Checking,
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

#[derive(Debug)]
pub enum TimeZoneCast {
  AsIs,
  Local,
  Utc,
}

impl From<manifest::TimeZoneCast> for TimeZoneCast {
  fn from(cast: manifest::TimeZoneCast) -> Self {
    match cast {
      manifest::TimeZoneCast::AsIs => Self::AsIs,
      manifest::TimeZoneCast::Local => Self::Local,
      manifest::TimeZoneCast::Utc => Self::Utc,
    }
  }
}

#[derive(Debug)]
pub struct App {
  pub calendar_state: crate::widget::calendar::CalendarState,
  pub difftool: String,
  pub modal: SelectedModal,
  pub repo_state: TableState,
  pub repos: IndexMap<String, Repo>,
  pub root_path: PathBuf,
  pub running: bool,
  pub selected_pane: SelectedPane,
  pub selected_repo_state: TableState,
  pub since: chrono::DateTime<chrono::Utc>,
  pub state: AppState,
  pub timezones: TimeZoneCast,
}

impl From<Manifest> for App {
  fn from(manifest: Manifest) -> Self {
    let repos: IndexMap<String, Repo> = manifest
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

    let since = manifest.since_datetime.unwrap();
    let calendar_state = CalendarState::from_datetime(&since);

    Self {
      calendar_state,
      repos,
      repo_state,
      selected_repo_state,
      since,
      difftool: manifest.difftool,
      modal: SelectedModal::default(),
      root_path: manifest.root.unwrap(),
      running: true,
      selected_pane: SelectedPane::default(),
      state: AppState::default(),
      timezones: manifest.timezones.into(),
    }
  }
}

impl App {
  pub fn tick(&mut self, sender: mpsc::Sender<Event>) -> AppResult<()> {
    if self.state == AppState::Init {
      self.update(sender)?;
      self.state = AppState::Checking;
    }
    Ok(())
  }

  pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
    let size = frame.size();
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

  pub fn reset(&mut self) {
    self.state = AppState::Init;
  }

  pub fn update(&self, sender: mpsc::Sender<Event>) -> AppResult<()> {
    for (id, repo) in &self.repos {
      repo.update(id.clone(), &self.root_path, sender.clone())?;
    }
    Ok(())
  }

  pub fn update_repo_status(&mut self, id: String, status: RepoStatus) -> AppResult<()> {
    if let Some(repo) = self.repos.get_mut(&id) {
      repo.status = status;
    }
    Ok(())
  }

  pub fn update_repo_logs(&mut self, id: String, logs: Vec<Log>) -> AppResult<()> {
    if let Some(repo) = self.repos.get_mut(&id) {
      repo.logs = logs;
      repo.status = RepoStatus::Finished;
    }

    self.repos.sort_by(&Self::sort_repos);

    Ok(())
  }

  #[allow(clippy::ptr_arg)]
  fn sort_repos(_key1: &String, repo1: &Repo, _key2: &String, repo2: &Repo) -> Ordering {
    if !repo1.logs.is_empty() && !repo2.logs.is_empty() {
      let commit1 = repo1.logs[0].cdate.clone();
      let commit2 = repo2.logs[0].cdate.clone();

      if commit1 > commit2 {
        return Ordering::Less;
      };
      if commit1 < commit2 {
        return Ordering::Greater;
      };
    };

    Ordering::Equal
  }
}
