use crate::app::{App, SelectedPane};
use crate::repo::{Repo, RepoStatus};

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
        let cells = [Cell::from(repo.name.clone()), status_icon(repo)];
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
    let text_style = Style::default()
        .fg(super::selected_color(app, SelectedPane::Repos))
        .add_modifier(Modifier::BOLD);
    text::Span::styled(" Repos ", text_style)
}

fn status_icon(repo: &Repo) -> Cell {
    let icon = match repo.status {
        RepoStatus::Checking => "⁇",
        RepoStatus::Cloning => "🐝",
        RepoStatus::Failed => "𝗫",
        RepoStatus::Finished => "✓",
        RepoStatus::Log => "🪵",
        RepoStatus::Pulling => "⤵",
    };
    Cell::from(icon)
}
