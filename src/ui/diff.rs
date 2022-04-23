use crate::app::{App, SelectedPane};

use tui::style::{Color, Modifier, Style};
use tui::text;
use tui::widgets::{Block, Borders};

pub fn render(app: &App) -> Block {
    Block::default()
        .title(title(app))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightCyan))
}

fn title(app: &App) -> text::Span {
    let text_style = Style::default()
        .fg(super::selected_color(app, SelectedPane::Diff))
        .add_modifier(Modifier::BOLD);
    text::Span::styled(" Diff ", text_style)
}
