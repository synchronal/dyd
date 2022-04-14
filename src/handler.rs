use crate::app::{App, AppResult, SelectedPane};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // navigate pane
        KeyCode::Left | KeyCode::Char('h') => match app.selected_pane {
            SelectedPane::Diff => (),
            SelectedPane::Stale => app.selected_pane = SelectedPane::Repos,
            _ => app.selected_pane = SelectedPane::Diff,
        },
        KeyCode::Right | KeyCode::Char('l') => match app.selected_pane {
            SelectedPane::Stale => (),
            SelectedPane::Repos => app.selected_pane = SelectedPane::Stale,
            _ => app.selected_pane = SelectedPane::Repos,
        },
        KeyCode::Tab => match app.selected_pane {
            SelectedPane::Diff => app.selected_pane = SelectedPane::Repos,
            SelectedPane::Repos => app.selected_pane = SelectedPane::Stale,
            SelectedPane::Stale => app.selected_pane = SelectedPane::Diff,
        },
        // exit application on ESC or q
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.running = false;
        }
        // exit application on Ctrl-D
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            }
        }
        _ => {}
    }
    Ok(())
}
