use crate::app::{App, AppResult, SelectedModal, SelectedPane};
use crate::git;
use crate::widget::calendar::CalendarState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
  match (&app.selected_pane, &app.modal, key_event.code) {
    // Calendar selection widget
    (_, SelectedModal::Calendar, KeyCode::Up | KeyCode::Char('k')) => decrement_calendar_week(app),
    (_, SelectedModal::Calendar, KeyCode::Down | KeyCode::Char('j')) => increment_calendar_week(app),
    (_, SelectedModal::Calendar, KeyCode::Left | KeyCode::Char('h')) => decrement_calendar_day(app),
    (_, SelectedModal::Calendar, KeyCode::Right | KeyCode::Char('l')) => increment_calendar_day(app),
    (_, SelectedModal::Calendar, KeyCode::Enter) => {
      select_calendar_day(app);
      close_modal(app);
    }

    // Scroll through lists
    (SelectedPane::Diff, _, KeyCode::Up | KeyCode::Char('k')) => decrement_selected_log(app, 1),
    (SelectedPane::Diff, _, KeyCode::Down | KeyCode::Char('j')) => increment_selected_log(app, 1),
    (SelectedPane::Diff, _, KeyCode::Char(' ') | KeyCode::Char('f')) => increment_selected_log(app, 10),
    (SelectedPane::Diff, _, KeyCode::Char('b')) => decrement_selected_log(app, 10),

    (SelectedPane::Repos, _, KeyCode::Up | KeyCode::Char('k')) => decrement_repos(app, 1),
    (SelectedPane::Repos, _, KeyCode::Down | KeyCode::Char('j')) => increment_repos(app, 1),
    (SelectedPane::Repos, _, KeyCode::Char(' ') | KeyCode::Char('f')) => increment_repos(app, 10),
    (SelectedPane::Repos, _, KeyCode::Char('b')) => decrement_repos(app, 10),

    // navigate pane
    (SelectedPane::Diff, _, KeyCode::Left | KeyCode::Char('h')) => (),
    (SelectedPane::Repos, _, KeyCode::Left | KeyCode::Char('h')) => app.selected_pane = SelectedPane::Diff,
    (SelectedPane::Diff, _, KeyCode::Right | KeyCode::Char('l')) => app.selected_pane = SelectedPane::Repos,
    (SelectedPane::Diff, _, KeyCode::Tab) => app.selected_pane = SelectedPane::Repos,
    (SelectedPane::Repos, _, KeyCode::Tab) => app.selected_pane = SelectedPane::Diff,

    // open diff
    (SelectedPane::Diff, _, KeyCode::Char('d')) => open_git_difftool(app),

    // update
    (SelectedPane::Diff, _, KeyCode::Char('r')) => app.reset(),
    (SelectedPane::Repos, _, KeyCode::Char('r')) => app.reset(),

    // exit application on ESC, q, or Ctrl-D
    (_, SelectedModal::None, KeyCode::Esc) => app.running = false,
    (_, _, KeyCode::Char('q') | KeyCode::Char('Q')) => app.running = false,
    (_, _, KeyCode::Char('d') | KeyCode::Char('D')) => {
      if key_event.modifiers == KeyModifiers::CONTROL {
        app.running = false;
      }
    }

    // modal management
    (_, _, KeyCode::Char('s')) => {
      if app.modal == SelectedModal::Calendar {
        close_modal(app)
      } else {
        app.calendar_state = CalendarState::from_datetime(&app.since);
        open_modal(app, SelectedModal::Calendar)
      }
    }
    (_, _, KeyCode::Esc) => close_modal(app),
    _ => {}
  }
  Ok(())
}

// // // Panes

fn decrement_selected_log(app: &mut App, count: usize) {
  if let Some(current) = app.selected_repo_state.selected() {
    if let Some(next_line) = current.checked_sub(count) {
      app.selected_repo_state.select(Some(next_line))
    } else {
      app.selected_repo_state.select(Some(0))
    };
  }
}

fn increment_selected_log(app: &mut App, count: usize) {
  let selected_repo_index = app.repo_state.selected().unwrap();
  let (_id, selected_repo) = app.repos.get_index(selected_repo_index).unwrap();

  let max_log: usize = selected_repo.logs.len() - 1;

  if let Some(current) = app.selected_repo_state.selected() {
    let next = std::cmp::min(current + count, max_log);
    app.selected_repo_state.select(Some(next));
  }
}

fn decrement_repos(app: &mut App, count: usize) {
  app.selected_repo_state.select(Some(0));
  if let Some(current) = app.repo_state.selected() {
    if let Some(next) = current.checked_sub(count) {
      app.repo_state.select(Some(next))
    } else {
      app.repo_state.select(Some(0))
    };
  }
}

fn increment_repos(app: &mut App, count: usize) {
  app.selected_repo_state.select(Some(0));
  let max_repos: usize = app.repos.len() - 1;
  if let Some(current) = app.repo_state.selected() {
    let next = std::cmp::min(current + count, max_repos);
    app.repo_state.select(Some(next));
  }
}

// // // Calendar

fn decrement_calendar_day(app: &mut App) {
  app.calendar_state.decrement(1);
}

fn decrement_calendar_week(app: &mut App) {
  app.calendar_state.decrement(7);
}

fn increment_calendar_day(app: &mut App) {
  app.calendar_state.increment(1);
}

fn increment_calendar_week(app: &mut App) {
  app.calendar_state.increment(7);
}

fn select_calendar_day(app: &mut App) {
  app.since = app.calendar_state.to_utc_datetime();
}

// // // Modals

fn open_modal(app: &mut App, modal: SelectedModal) {
  app.modal = modal;
}

fn close_modal(app: &mut App) {
  app.modal = SelectedModal::None;
}

// // // Diff

fn open_git_difftool(app: &App) {
  let selected_repo_index = app.repo_state.selected().unwrap();
  let (_id, selected_repo) = app.repos.get_index(selected_repo_index).unwrap();
  let selected_log = app.selected_repo_state.selected().unwrap();
  let log = &selected_repo.logs[selected_log];

  git::open_difftool(&app.root_path, &app.difftool, selected_repo, log);
}
