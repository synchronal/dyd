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
        rows = selected_repo
            .logs
            .iter()
            .map(|log| {
                let cells = [
                    Cell::from(sha(&log.sha)),
                    Cell::from(age(&log.age)),
                    Cell::from(author(&log.author)),
                    Cell::from(message(&log.message)),
                ];
                Row::new(cells)
            })
            .collect()
    }

    Table::new(rows)
        .block(container)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("·")
        .column_spacing(2)
        .widths(&[
            Constraint::Length(9),
            Constraint::Length(17),
            Constraint::Percentage(20),
            Constraint::Percentage(100),
        ])
}

fn title(app: &App) -> text::Span {
    let text_style = Style::default()
        .fg(super::selected_color(app, SelectedPane::Diff))
        .add_modifier(Modifier::BOLD);
    text::Span::styled(" Diff ", text_style)
}

fn age(text: &String) -> text::Span {
    let text_style = Style::default().fg(Color::Red);
    text::Span::styled(text, text_style)
}

fn author(text: &String) -> text::Span {
    let text_style = Style::default().fg(Color::Yellow);
    text::Span::styled(text, text_style)
}

fn message(text: &String) -> text::Span {
    let text_style = Style::default().fg(Color::White);
    text::Span::styled(text, text_style)
}

fn sha(text: &String) -> text::Span {
    let text_style = Style::default();
    text::Span::styled(text, text_style)
}
