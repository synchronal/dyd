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
  use crate::manifest::{Manifest, Remote};
  use crate::theme::{ColorTheme, Theme};
  use crossterm::event::KeyEvent;

  fn app(remotes: &[&str]) -> App {
    let manifest = Manifest {
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
      root: Some("/nonexistent".into()),
      since_datetime: Some(chrono::Utc::now()),
      ..Default::default()
    };
    App::from_manifest(manifest, ColorTheme::try_from(Theme::Dark).unwrap())
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
  fn open_difftool_with_stale_log_selection_does_not_panic() {
    let mut app = app(&["one"]);
    app.selected_pane = SelectedPane::Diff;
    app.selected_repo_state.select(Some(5));
    press(&mut app, KeyCode::Char('d'));
  }
}
