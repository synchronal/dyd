use crate::manifest::Manifest;
use crate::repo::Repo;
use crate::ui;

use std::error;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::terminal::Frame;
use tui::text;
use tui::widgets::{Block, Borders, TableState};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Selected pane
#[derive(Debug, PartialEq)]
pub enum SelectedPane {
    Diff,
    Repos,
    Stale,
}

impl Default for SelectedPane {
    fn default() -> Self {
        SelectedPane::Repos
    }
}

#[derive(Debug, Default)]
pub struct App {
    pub repos: Vec<Repo>,
    pub repo_state: TableState,
    pub running: bool,
    pub selected_pane: SelectedPane,
    pub since: String,
}

impl From<Manifest> for App {
    fn from(manifest: Manifest) -> Self {
        let repos: Vec<Repo> = manifest
            .remotes
            .into_iter()
            .map(|(_, remote)| remote.into())
            .collect();

        let mut repo_state = TableState::default();
        if repos.len() > 0 {
            repo_state.select(Some(0))
        }

        Self {
            repos,
            repo_state,
            since: manifest.since,
            running: true,
            ..Default::default()
        }
    }
}

impl App {
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
        frame.render_widget(self.stale(), sidebar[1]);
        frame.render_widget(self.help(), sidebar[2]);
        frame.render_stateful_widget(
            ui::repos::render(self),
            sidebar[0],
            &mut self.repo_state.clone(),
        );
    }
    fn diff(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Diff ",
                Style::default().fg(ui::selected_color(self, SelectedPane::Diff)),
            ))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::LightCyan))
    }

    fn stale(&self) -> Block {
        Block::default()
            .title(text::Span::styled(
                " Stale ",
                Style::default().fg(ui::selected_color(self, SelectedPane::Stale)),
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
}
