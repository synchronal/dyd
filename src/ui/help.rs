use crate::app::App;

use tui::style::{Color, Modifier, Style};
use tui::text;
use tui::widgets::{Block, Borders};

pub fn render(app: &App) -> Block {
    Block::default()
        .title(title(app))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightCyan))
}

fn title(_app: &App) -> text::Span {
    let text_style = Style::default().fg(Color::Gray).add_modifier(Modifier::DIM);
    text::Span::styled(" Help ", text_style)
}
