use crate::app::{App, SelectedPane};

use ratatui::layout::Constraint;
use ratatui::style::{Modifier, Style};
use ratatui::text;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

pub fn render(app: &App) -> Table {
  let container = Block::default()
    .title(title(app))
    .borders(Borders::ALL)
    .style(Style::default().fg(app.theme.border_color()));

  let mut rows: Vec<Row> = vec![];

  if let Some(index) = app.repo_state.selected() {
    let (_id, selected_repo) = app.repos.get_index(index).unwrap();
    rows = selected_repo
      .logs
      .iter()
      .map(|log| {
        let stale = app.since >= log.commit_datetime;

        let cells = [
          Cell::from(sha(&log.sha, app)),
          Cell::from(age(&log.commit_datetime, app)),
          Cell::from(author(&log.author, app)),
          Cell::from(message(&log.message, app)),
        ];

        Row::new(cells).style(stale_style(stale))
      })
      .collect()
  }
  let widths = [
    Constraint::Length(9),
    Constraint::Length(17),
    Constraint::Percentage(20),
    Constraint::Percentage(100),
  ];

  Table::new(rows, widths)
    .block(container)
    .row_highlight_style(app.theme.diff_row_hightlight_style())
    .highlight_symbol("·")
    .column_spacing(2)
}

fn title(app: &App) -> text::Span {
  let text_style = Style::default()
    .fg(super::selected_color(app, SelectedPane::Diff))
    .add_modifier(Modifier::BOLD);
  text::Span::styled(" Diff ", text_style)
}

fn age<'a>(datetime: &'a chrono::DateTime<chrono::Utc>, app: &App) -> text::Span<'a> {
  let text_style = Style::default().fg(app.theme.diff_age_color());
  let text = datetime
    .with_timezone(&app.timezone_offset)
    .format("%a %b %d %R");
  text::Span::styled(text.to_string(), text_style)
}

fn author<'a>(text: &'a String, app: &'a App) -> text::Span<'a> {
  let text_style = Style::default().fg(app.theme.diff_author_color());
  text::Span::styled(text, text_style)
}

fn message<'a>(text: &'a String, app: &'a App) -> text::Span<'a> {
  let text_style = Style::default().fg(app.theme.diff_message_color());
  text::Span::styled(text, text_style)
}

fn sha<'a>(text: &'a String, app: &'a App) -> text::Span<'a> {
  let text_style = Style::default().fg(app.theme.diff_sha_color());
  text::Span::styled(text, text_style)
}

fn stale_style(stale: bool) -> Style {
  if stale {
    Style::default().add_modifier(Modifier::DIM)
  } else {
    Style::default()
  }
}
