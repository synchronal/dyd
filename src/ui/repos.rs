use crate::app::{App, SelectedPane};

use tui::layout::Constraint;
use tui::style::{Color, Modifier, Style};
use tui::text;
use tui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render(app: &App) -> Table {
    let container = Block::default()
        .title(title(app))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightCyan));

    let rows = app.repos.iter().map(|repo| {
        let cells = [Cell::from(repo.name.clone()), Cell::from("✓")];
        Row::new(cells)
    });

    Table::new(rows)
        .block(container)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("·")
        .column_spacing(2)
        .widths(&[Constraint::Percentage(93), Constraint::Length(1)])
}

fn title(app: &App) -> text::Span {
    let text_style = Style::default().fg(super::selected_color(app, SelectedPane::Repos));
    text::Span::styled(" Repos ", text_style)
}
