use crate::app::{App, AppResult, SelectedModal, SelectedPane};
use crate::git::repo::Repo;
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
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Up | KeyCode::Char('k')) => decrement_selected_log(app, 1),
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Down | KeyCode::Char('j')) => increment_selected_log(app, 1),
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Char(' ') | KeyCode::Char('f')) => {
      increment_selected_log(app, 10)
    }
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Char('b')) => decrement_selected_log(app, 10),

    (SelectedPane::Repos, SelectedModal::None, KeyCode::Up | KeyCode::Char('k')) => decrement_repos(app, 1),
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Down | KeyCode::Char('j')) => increment_repos(app, 1),
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Char(' ') | KeyCode::Char('f')) => increment_repos(app, 10),
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Char('b')) => decrement_repos(app, 10),

    // navigate pane
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Left | KeyCode::Char('h')) => (),
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Left | KeyCode::Char('h')) => {
      app.selected_pane = SelectedPane::Diff
    }
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Right | KeyCode::Char('l')) => {
      app.selected_pane = SelectedPane::Repos
    }
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Tab) => app.selected_pane = SelectedPane::Repos,
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Tab) => app.selected_pane = SelectedPane::Diff,

    // open diff
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Char('d')) if key_event.modifiers != KeyModifiers::CONTROL => {
      open_git_difftool(app)
    }

    // update
    (SelectedPane::Diff, SelectedModal::None, KeyCode::Char('r')) => app.request_refresh(),
    (SelectedPane::Repos, SelectedModal::None, KeyCode::Char('r')) => app.request_refresh(),

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
        app.calendar_state = (&app.since).into();
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
  let Some(max_log) = selected_repo(app).and_then(|repo| repo.logs.len().checked_sub(1)) else {
    return;
  };

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
  let Some(max_repos) = app.repos.len().checked_sub(1) else {
    return;
  };
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
  let Some(selected_repo) = selected_repo(app) else {
    return;
  };
  let Some(log) = app
    .selected_repo_state
    .selected()
    .and_then(|index| selected_repo.logs.get(index))
  else {
    return;
  };

  app.difftool.open(&app.root_path, selected_repo, log);
}

fn selected_repo(app: &App) -> Option<&Repo> {
  let index = app.repo_state.selected()?;
  app.repos.get_index(index).map(|(_id, repo)| repo)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::app::AppState;
  use crate::config::{Config, Remote};
  use crate::difftool::Difftool;
  use crate::theme::{ColorTheme, Theme};
  use crossterm::event::KeyEvent;

  fn app(remotes: &[&str]) -> App {
    let config = Config {
      difftool: Difftool::Git,
      remotes: remotes
        .iter()
        .map(|id| {
          let remote = Remote {
            name: id.to_string(),
            origin: format!("git@github.com:example/{id}.git"),
            branch: None,
          };
          (id.to_string(), remote)
        })
        .collect(),
      root: "/nonexistent".into(),
      since: chrono::Utc::now(),
      theme: Theme::Dark,
    };
    App::from_config(config, ColorTheme::try_from(Theme::Dark).unwrap())
  }

  fn press(app: &mut App, code: KeyCode) {
    handle_key_events(KeyEvent::from(code), app).unwrap();
  }

  #[test]
  fn keys_without_repos_do_not_panic() {
    let mut app = app(&[]);
    for pane in [SelectedPane::Repos, SelectedPane::Diff] {
      app.selected_pane = pane;
      for code in [
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::Char('f'),
        KeyCode::Char('b'),
        KeyCode::Char('d'),
      ] {
        press(&mut app, code);
      }
    }
  }

  #[test]
  fn keys_before_logs_load_do_not_panic() {
    let mut app = app(&["one", "two"]);
    app.selected_pane = SelectedPane::Diff;
    for code in [KeyCode::Down, KeyCode::Char('f'), KeyCode::Char('d')] {
      press(&mut app, code);
    }
    assert_eq!(app.selected_repo_state.selected(), Some(0));
  }

  #[test]
  fn refresh_ignored_until_initial_refresh_finishes() {
    let mut app = app(&["one"]);
    assert_eq!(app.state, AppState::Idle);

    app.request_refresh();
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.state, AppState::RefreshRequested);

    // What tick does, without spawning a real clone.
    let (sender, _receiver) = std::sync::mpsc::channel();
    app.state = AppState::Idle;
    let task = app.refresh.track(sender);
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.state, AppState::Idle);

    // The update thread has exited, but the loop has not processed its done event.
    drop(task);
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.state, AppState::Idle);

    app.finish_repo_update();
    press(&mut app, KeyCode::Char('r'));
    assert_eq!(app.state, AppState::RefreshRequested);
  }

  #[test]
  fn pane_keys_ignored_while_calendar_open() {
    let mut app = app(&["one", "two", "three"]);
    for pane in [SelectedPane::Repos, SelectedPane::Diff] {
      app.selected_pane = pane;
      press(&mut app, KeyCode::Char('s'));
      for code in [
        KeyCode::Char('f'),
        KeyCode::Char(' '),
        KeyCode::Char('b'),
        KeyCode::Char('r'),
        KeyCode::Tab,
      ] {
        press(&mut app, code);
      }
      assert_eq!(app.modal, SelectedModal::Calendar);
      assert_eq!(app.repo_state.selected(), Some(0));
      assert_eq!(app.state, AppState::Idle);

      press(&mut app, KeyCode::Esc);
      assert_eq!(app.modal, SelectedModal::None);
    }
    assert_eq!(app.selected_pane, SelectedPane::Diff);
  }

  #[test]
  fn ctrl_d_quits_from_every_pane() {
    for pane in [SelectedPane::Repos, SelectedPane::Diff] {
      let mut app = app(&["one"]);
      app.selected_pane = pane;
      handle_key_events(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL), &mut app).unwrap();
      assert!(!app.running);
    }
  }

  #[test]
  fn open_difftool_with_stale_log_selection_does_not_panic() {
    let mut app = app(&["one"]);
    app.selected_pane = SelectedPane::Diff;
    app.selected_repo_state.select(Some(5));
    press(&mut app, KeyCode::Char('d'));
  }
}
