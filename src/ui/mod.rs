use crate::app::{App, SelectedPane};
use ratatui::style::Color;

pub mod diff;
pub mod help;
pub mod modal;
pub mod repos;

pub fn selected_color(app: &App, pane: SelectedPane) -> Color {
  if pane == app.selected_pane {
    app.theme.header_selected_color
  } else {
    Color::Reset
  }
}
