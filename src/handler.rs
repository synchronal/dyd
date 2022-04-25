use crate::app::{App, AppResult, SelectedPane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match (&app.selected_pane, key_event.code) {
        (SelectedPane::Repos, KeyCode::Up | KeyCode::Char('k')) => decrement_repos(app),
        (SelectedPane::Repos, KeyCode::Down | KeyCode::Char('j')) => increment_repos(app),
        // navigate pane
        (SelectedPane::Diff, KeyCode::Left | KeyCode::Char('h')) => (),
        (SelectedPane::Repos, KeyCode::Left | KeyCode::Char('h')) => app.selected_pane = SelectedPane::Diff,
        (SelectedPane::Stale, KeyCode::Left | KeyCode::Char('h')) => app.selected_pane = SelectedPane::Repos,
        (SelectedPane::Diff, KeyCode::Right | KeyCode::Char('l')) => app.selected_pane = SelectedPane::Repos,
        (SelectedPane::Repos, KeyCode::Right | KeyCode::Char('l')) => app.selected_pane = SelectedPane::Stale,
        (SelectedPane::Stale, KeyCode::Right | KeyCode::Char('l')) => (),
        (SelectedPane::Diff, KeyCode::Tab) => app.selected_pane = SelectedPane::Repos,
        (SelectedPane::Repos, KeyCode::Tab) => app.selected_pane = SelectedPane::Stale,
        (SelectedPane::Stale, KeyCode::Tab) => app.selected_pane = SelectedPane::Diff,
        // exit application on ESC or q
        (_, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q')) => app.running = false,
        // exit application on Ctrl-D
        (_, KeyCode::Char('d') | KeyCode::Char('D')) => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        _ => {}
    }
    Ok(())
}

fn decrement_repos(app: &mut App) {
    app.selected_repo_state.select(Some(0));
    if let Some(current) = app.repo_state.selected() {
        if current > 0 {
            app.repo_state.select(Some(current - 1))
        };
    }
}

fn increment_repos(app: &mut App) {
    app.selected_repo_state.select(Some(0));
    let max_repos: usize = app.repos.len() - 1;
    if let Some(current) = app.repo_state.selected() {
        if current < max_repos {
            app.repo_state.select(Some(current + 1))
        };
    }
}
