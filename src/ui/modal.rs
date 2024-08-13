use crate::app::{App, SelectedModal};
use crate::widget::calendar::Calendar;
use tui::layout::{Margin, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Clear};
use tui::Frame;

pub fn render(app: &App, frame: &mut Frame) {
  let window = frame.area();

  match app.modal {
    SelectedModal::None => {}
    SelectedModal::Calendar => {
      let x = std::cmp::max(window.width / 2 - 12, 0);
      let y = std::cmp::max(window.height / 2 - 10, 0);
      let background = Rect::new(x, y, 28, 12);
      let area = background.inner(Margin {
        vertical: 1,
        horizontal: 2,
      });

      let container = Block::default()
        .title(" Calendar ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::LightCyan));

      let calendar = Calendar::new().block(container);

      frame.render_widget(Clear, background);
      frame.render_stateful_widget(calendar, area, &mut app.calendar_state.clone());
    }
  };
}
