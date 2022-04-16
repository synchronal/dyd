use crate::app::{App, AppResult, SelectedPane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match (&app.selected_pane, key_event.code) {
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
