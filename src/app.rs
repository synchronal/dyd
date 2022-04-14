use std::error;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text;
use tui::widgets::{Block, Borders};

use crate::manifest::Manifest;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Selected pane
#[derive(Debug, PartialEq)]
pub enum SelectedPane {
    Diff,
    Repos,
    Stale,
}

// impl From<SelectedPane> for usize {
//     fn from(input: SelectedPane) -> usize {
//         match input {
//             SelectedPane::Diff => 0,
//             SelectedPane::Repos => 1,
//             SelectedPane::Stale => 2,
//         }
//     }
// }

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub manifest: Manifest,
    pub selected_pane: SelectedPane,
}

impl App {
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            running: true,
            selected_pane: SelectedPane::Repos,
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

        let sidebar = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints(
                [
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Ratio(1, 3),
                ]
                .as_ref(),
            )
            .split(layout[1]);

        frame.render_widget(self.diff(), layout[0]);
        frame.render_widget(self.repos(), sidebar[0]);
        frame.render_widget(self.stale(), sidebar[1]);
        frame.render_widget(self.help(), sidebar[2]);
    }
    fn diff(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Diff ",
                Style::default().fg(self.selected_color(SelectedPane::Diff)),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
    }

    fn repos(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Repos ",
                Style::default().fg(self.selected_color(SelectedPane::Repos)),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
    }

    fn stale(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Stale ",
                Style::default().fg(self.selected_color(SelectedPane::Stale)),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
    }

    fn help(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Help ",
                Style::default().fg(Color::White),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
    }

    fn selected_color(&self, pane: SelectedPane) -> Color {
        if pane == self.selected_pane {
            Color::Red
        } else {
            Color::White
        }
    }
}
