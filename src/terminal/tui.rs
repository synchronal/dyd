use crate::app::{App, AppResult, EventHandler};
use crossterm::cursor::Show;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal,
/// initializing the interface and handling the draw events.
#[derive(Debug)]
pub struct Tui<B: Backend> {
  /// Interface to the Terminal.
  terminal: Terminal<B>,
  /// Terminal event handler.
  pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
  /// Constructs a new instance of [`Tui`].
  pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
    Self { terminal, events }
  }

  /// Initializes the terminal interface.
  ///
  /// It enables the raw mode and sets terminal properties.
  pub fn init(&mut self) -> AppResult<()>
  where
    <B as Backend>::Error: 'static,
  {
    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
    match self.terminal.hide_cursor() {
      Ok(_it) => Ok(()),
      Err(err) => Err(Box::new(err)),
    }
  }

  /// [`Draw`] the terminal interface by [`rendering`] the widgets.
  ///
  /// [`Draw`]: tui::Terminal::draw
  /// [`rendering`]: crate::app::App::render
  pub fn draw(&mut self, app: &mut App) -> AppResult<()>
  where
    <B as Backend>::Error: 'static,
  {
    match self.terminal.draw(|frame| app.render(frame)) {
      Ok(_it) => Ok(()),
      Err(err) => Err(Box::new(err)),
    }
  }

  /// Exits the terminal interface.
  ///
  /// It disables the raw mode and reverts back the terminal properties.
  pub fn exit(&mut self) -> AppResult<()>
  where
    <B as Backend>::Error: 'static,
  {
    restore()?;
    match self.terminal.show_cursor() {
      Ok(_it) => Ok(()),
      Err(err) => Err(Box::new(err)),
    }
  }
}

/// Reverts the terminal properties without a [`Tui`], for the panic hook.
pub fn restore() -> io::Result<()> {
  terminal::disable_raw_mode()?;
  crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture, Show)
}

/// Restores the terminal before reporting a main-thread panic, so the message
/// is readable and the shell usable. Worker-thread panics leave the interface
/// running.
pub fn install_panic_hook() {
  let default_hook = std::panic::take_hook();
  std::panic::set_hook(Box::new(move |info| {
    if std::thread::current().name() == Some("main") {
      let _ = restore();
    }
    default_hook(info);
  }));
}
