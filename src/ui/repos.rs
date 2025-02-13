use crate::app::{App, SelectedPane};
use crate::git::repo::{Repo, RepoStatus};

use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render(app: &App) -> Table {
  let container = Block::default()
    .title(title(app))
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::LightCyan));

  let rows = app.repos.iter().map(|(_id, repo)| {
    let cells = [status_icon(repo), Cell::from(format!("{repo}"))];
    Row::new(cells)
  });

  let widths = [Constraint::Length(2), Constraint::Percentage(100)];

  Table::new(rows, widths)
    .block(container)
    .row_highlight_style(Style::default().add_modifier(Modifier::BOLD))
    .highlight_symbol("·")
    .column_spacing(2)
}

fn title(app: &App) -> text::Span {
  let text_style = Style::default()
    .fg(super::selected_color(app, SelectedPane::Repos))
    .add_modifier(Modifier::BOLD);
  text::Span::styled(" Repos ", text_style)
}

fn status_icon(repo: &Repo) -> Cell {
  let icon = match repo.status {
    RepoStatus::Checking => " ⁇",
    RepoStatus::Cloning => " ⚭",
    RepoStatus::Failed => " 𝗫",
    RepoStatus::Finished => " ✓",
    RepoStatus::Log => " ☈",
    RepoStatus::Pulling => " ⤵",
  };
  Cell::from(icon)
}
