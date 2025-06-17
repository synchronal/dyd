use crate::app::AppResult;
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use terminal_colorsaurus::{color_palette, ColorScheme, QueryOptions};

#[derive(clap:: ValueEnum, Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
  #[default]
  Auto,
  Dark,
  Light,
}

impl Theme {
  pub fn consume(self) -> AppResult<Box<dyn ColorTheme>> {
    match self {
      Theme::Auto => detect_colortheme(),
      Theme::Dark => Ok(Box::new(DarkTheme)),
      Theme::Light => Ok(Box::new(LightTheme)),
    }
  }
}

fn detect_colortheme() -> AppResult<Box<dyn ColorTheme>> {
  let colors = color_palette(QueryOptions::default())?;
  match colors.color_scheme() {
    ColorScheme::Dark => Ok(Box::new(DarkTheme)),
    ColorScheme::Light => Ok(Box::new(LightTheme)),
  }
}

pub trait ColorTheme: std::fmt::Debug {
  fn border_color(&self) -> Color;
  fn diff_age_color(&self) -> Color;
  fn diff_author_color(&self) -> Color;
  fn diff_message_color(&self) -> Color;
  fn diff_row_hightlight_style(&self) -> Style;
  fn diff_sha_color(&self) -> Color;
  fn header_selected_color(&self) -> Color;
  fn help_header_style(&self) -> Style;
  fn help_text_style(&self) -> Style;
  fn repo_row_hightlight_style(&self) -> Style;
  fn text_color(&self) -> Color;
}

#[derive(Debug)]
pub struct DarkTheme;
#[derive(Debug)]
pub struct LightTheme;

impl ColorTheme for DarkTheme {
  fn border_color(&self) -> Color {
    Color::LightCyan
  }
  fn diff_age_color(&self) -> Color {
    Color::Red
  }
  fn diff_author_color(&self) -> Color {
    Color::Yellow
  }
  fn diff_message_color(&self) -> Color {
    Color::White
  }
  fn diff_row_hightlight_style(&self) -> Style {
    Style::default().add_modifier(Modifier::BOLD)
  }
  fn diff_sha_color(&self) -> Color {
    Color::LightCyan
  }
  fn header_selected_color(&self) -> Color {
    Color::Red
  }
  fn help_header_style(&self) -> Style {
    Style::default().fg(Color::Gray).add_modifier(Modifier::DIM)
  }
  fn help_text_style(&self) -> Style {
    Style::default().fg(Color::Cyan)
  }
  fn repo_row_hightlight_style(&self) -> Style {
    Style::default().add_modifier(Modifier::BOLD)
  }
  fn text_color(&self) -> Color {
    Color::LightCyan
  }
}

impl ColorTheme for LightTheme {
  fn border_color(&self) -> Color {
    Color::Cyan
  }
  fn diff_age_color(&self) -> Color {
    Color::Red
  }
  fn diff_author_color(&self) -> Color {
    Color::Blue
  }
  fn diff_message_color(&self) -> Color {
    Color::Black
  }
  fn diff_row_hightlight_style(&self) -> Style {
    Style::default()
      .add_modifier(Modifier::UNDERLINED)
      .add_modifier(Modifier::BOLD)
  }
  fn diff_sha_color(&self) -> Color {
    Color::Magenta
  }
  fn header_selected_color(&self) -> Color {
    Color::Red
  }
  fn help_header_style(&self) -> Style {
    Style::default()
      .fg(Color::Black)
      .add_modifier(Modifier::DIM)
  }
  fn help_text_style(&self) -> Style {
    Style::default()
      .fg(Color::Black)
      .add_modifier(Modifier::DIM)
  }
  fn repo_row_hightlight_style(&self) -> Style {
    Style::default()
      .add_modifier(Modifier::UNDERLINED)
      .add_modifier(Modifier::BOLD)
  }
  fn text_color(&self) -> Color {
    Color::Black
  }
}
