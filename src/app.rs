use crate::manifest::Manifest;
use crate::repo::Repo;
use crate::ui;

use std::collections::HashMap;
use std::error;
use std::path::PathBuf;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::terminal::Frame;
use tui::widgets::TableState;

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
    pub repos: HashMap<String, Repo>,
    pub repo_state: TableState,
    pub running: bool,
    pub root_path: PathBuf,
    pub selected_pane: SelectedPane,
    pub since: String,
}

impl From<Manifest> for App {
    fn from(manifest: Manifest) -> Self {
        let repos: HashMap<String, Repo> = manifest
            .remotes
            .into_iter()
            .map(|(id, remote)| (id, remote.into()))
            .collect();

        let mut repo_state = TableState::default();
        if repos.len() > 0 {
            repo_state.select(Some(0))
        }

        Self {
            repos,
            repo_state,
            root_path: manifest.root.unwrap(),
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

        frame.render_widget(ui::diff::render(self), layout[0]);
        frame.render_widget(ui::stale::render(self), sidebar[1]);
        frame.render_widget(ui::help::render(self), sidebar[2]);
        frame.render_stateful_widget(ui::repos::render(self), sidebar[0], &mut self.repo_state.clone());
    }

    pub fn update(&self) -> AppResult<()> {
        for (id, repo) in &self.repos {
            repo.update(id, &self.root_path)?;
        }
        Ok(())
    }
}
