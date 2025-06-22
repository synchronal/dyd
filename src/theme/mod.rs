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
  pub fn consume(self) -> AppResult<ColorTheme> {
    match self {
      Theme::Auto => detect_colortheme(),
      Theme::Dark => Ok(dark_theme()),
      Theme::Light => Ok(light_theme()),
    }
  }
}

fn detect_colortheme() -> AppResult<ColorTheme> {
  let colors = color_palette(QueryOptions::default())?;
  match colors.color_scheme() {
    ColorScheme::Dark => Ok(dark_theme()),
    ColorScheme::Light => Ok(light_theme()),
  }
}

#[derive(Debug)]
pub struct ColorTheme {
  pub border_color: Color,
  pub diff_age_color: Color,
  pub diff_author_color: Color,
  pub diff_message_color: Color,
  pub diff_row_hightlight_style: Style,
  pub diff_sha_color: Color,
  pub header_selected_color: Color,
  pub help_header_style: Style,
  pub help_text_style: Style,
  pub repo_row_hightlight_style: Style,
  pub text_color: Color,
}

fn dark_theme() -> ColorTheme {
  ColorTheme {
    border_color: Color::LightCyan,
    diff_age_color: Color::Red,
    diff_author_color: Color::Yellow,
    diff_message_color: Color::White,
    diff_row_hightlight_style: Style::default().add_modifier(Modifier::BOLD),
    diff_sha_color: Color::LightCyan,
    header_selected_color: Color::Red,
    help_header_style: Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
    help_text_style: Style::default().fg(Color::Cyan),
    repo_row_hightlight_style: Style::default().add_modifier(Modifier::BOLD),
    text_color: Color::LightCyan,
  }
}

fn light_theme() -> ColorTheme {
  ColorTheme {
    border_color: Color::Cyan,
    diff_age_color: Color::Red,
    diff_author_color: Color::Blue,
    diff_message_color: Color::Black,
    diff_row_hightlight_style: Style::default()
      .add_modifier(Modifier::UNDERLINED)
      .add_modifier(Modifier::BOLD),
    diff_sha_color: Color::Magenta,
    header_selected_color: Color::Red,
    help_header_style: Style::default()
      .fg(Color::Black)
      .add_modifier(Modifier::DIM),
    help_text_style: Style::default()
      .fg(Color::Black)
      .add_modifier(Modifier::DIM),
    repo_row_hightlight_style: Style::default()
      .add_modifier(Modifier::UNDERLINED)
      .add_modifier(Modifier::BOLD),
    text_color: Color::Black,
  }
}
