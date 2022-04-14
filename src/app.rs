use std::error;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text;
use tui::widgets::{Block, Borders};

use crate::manifest::Manifest;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub manifest: Manifest,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            running: true,
        }
    }

    pub fn tick(&self) {}

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<'_, B>) {
        let size = frame.size();
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(0)
            .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
            .split(size);

        let diff = Block::default()
            .title(text::Span::styled(
                " Diff ",
                Style::default().fg(Color::White),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan));

        let _sidebar = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(layout[1]);

        let _repos = Block::default()
            .title(text::Span::styled("Repos", Style::default().fg(Color::Red)))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan));

        frame.render_widget(diff, layout[0]);
    }
}
