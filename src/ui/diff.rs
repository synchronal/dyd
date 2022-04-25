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

    let mut rows: Vec<Row> = vec![];

    if let Some(index) = app.repo_state.selected() {
        let (_id, selected_repo) = app.repos.get_index(index).unwrap();
        rows = selected_repo.logs.iter().map(|log| {
            let cells = [
                Cell::from(log.sha.clone()),
                Cell::from(log.age.clone()),
                Cell::from(log.author.clone()),
                Cell::from(log.message.clone()),
            ];
            Row::new(cells)
        }).collect()
    }

    Table::new(rows)
        .block(container)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("·")
        .column_spacing(2)
        .widths(&[
            Constraint::Length(9),
            Constraint::Length(17),
            Constraint::Percentage(25),
            Constraint::Percentage(100),
        ])
}

fn title(app: &App) -> text::Span {
    let text_style = Style::default()
        .fg(super::selected_color(app, SelectedPane::Diff))
        .add_modifier(Modifier::BOLD);
    text::Span::styled(" Diff ", text_style)
}
