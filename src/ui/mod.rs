use crate::app::{App, SelectedPane};

use tui::style::Color;

pub mod diff;
pub mod help;
pub mod repos;
pub mod stale;

pub fn selected_color(app: &App, pane: SelectedPane) -> Color {
    if pane == app.selected_pane {
        Color::Red
    } else {
        Color::Reset
    }
}
