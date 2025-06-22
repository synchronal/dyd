use crate::app::App;

use ratatui::style::Style;
use ratatui::text::{self, Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

const VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub fn render(app: &App) -> Paragraph {
  let text = vec![
    Line::from(vec![
      Span::raw(" hl ←→ "),
      Span::raw(" — "),
      Span::raw("navigate panes"),
    ]),
    Line::from(vec![
      Span::raw(" <tab> "),
      Span::raw(" — "),
      Span::raw("cycle through panes"),
    ]),
    Line::from(vec![
      Span::raw(" jk ↑↓ "),
      Span::raw(" — "),
      Span::raw("next / previous"),
    ]),
    Line::from(vec![Span::raw(" f␣    "), Span::raw(" — "), Span::raw("page forward")]),
    Line::from(vec![
      Span::raw(" b     "),
      Span::raw(" — "),
      Span::raw("page backwards"),
    ]),
    Line::from(vec![
      Span::raw(" d     "),
      Span::raw(" — "),
      Span::raw("open git difftool"),
    ]),
    Line::from(vec![Span::raw(" r     "), Span::raw(" — "), Span::raw("refresh repos")]),
    Line::from(vec![Span::raw("   ")]),
    Line::from(vec![
      Span::raw(" s     "),
      Span::raw(" — "),
      Span::raw("open / close calendar"),
    ]),
    Line::from(vec![Span::raw(" <enter>"), Span::raw(" — "), Span::raw("select date")]),
    Line::from(vec![Span::raw(" <esc>  "), Span::raw(" — "), Span::raw("close modal")]),
    Line::from(vec![Span::raw("   ")]),
    Line::from(vec![Span::raw(" q <esc>"), Span::raw(" — "), Span::raw("quit")]),
  ];

  Paragraph::new(text)
    .block(
      Block::default()
        .title(title(app))
        .borders(Borders::ALL)
        .style(Style::default().fg(app.theme.border_color)),
    )
    .style(app.theme.help_text_style)
}

fn title(app: &App) -> text::Span {
  text::Span::styled(format!(" Help (v{})", VERSION.unwrap()), app.theme.help_header_style)
}
