use chrono::{DateTime, Datelike, Duration, NaiveDate, Timelike, Utc, Weekday};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, StatefulWidget, Widget};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Calendar<'a> {
  block: Option<Block<'a>>,
  style: Style,
}

impl<'a> Calendar<'a> {
  pub fn new() -> Self {
    Self {
      block: None,
      style: Default::default(),
    }
  }

  pub fn block(mut self, block: Block<'a>) -> Self {
    self.block = Some(block);
    self
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarState {
  datetime: NaiveDate,
}

impl CalendarState {
  pub fn from_datetime(datetime: &DateTime<Utc>) -> Self {
    Self {
      datetime: datetime.date_naive(),
    }
  }

  pub fn today() -> Self {
    Self {
      datetime: chrono::Utc::now().date_naive(),
    }
  }

  pub fn to_utc_datetime(&self) -> DateTime<Utc> {
    let mut datetime = chrono::Utc::now();
    datetime = datetime
      .with_hour(0)
      .unwrap()
      .with_minute(0)
      .unwrap()
      .with_second(0)
      .unwrap();

    datetime = datetime
      .with_year(self.datetime.year())
      .unwrap()
      .with_month(self.datetime.month())
      .unwrap()
      .with_day(self.datetime.day())
      .unwrap();

    datetime
  }

  pub fn decrement(&mut self, days: i64) {
    self.datetime -= Duration::try_days(days).unwrap();
  }

  pub fn increment(&mut self, days: i64) {
    self.datetime += Duration::try_days(days).unwrap();
  }
}

impl<'a> StatefulWidget for Calendar<'a> {
  type State = CalendarState;

  fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
    if area.width < 22 || area.height < 8 {
      return;
    }

    buf.set_style(area, self.style);
    let calendar_area = match self.block.take() {
      Some(b) => {
        let inner_area = b.inner(area);
        b.render(area, buf);
        inner_area
      }
      None => area,
    };

    overheader(buf, calendar_area, state.datetime);
    header(buf, calendar_area, state.datetime);

    let today = chrono::Utc::now().date_naive();
    let days_in_month = total_days_in_month(state.datetime.year(), state.datetime.month()).unwrap();
    let mut current_date = state.datetime.with_day(1).unwrap();

    let mut printing_week = 1;
    let mut printing = true;

    while printing {
      let day_of_week = current_date.weekday();
      let day = current_date.day();
      let y = area.y + 2 + printing_week;
      let x = area.x + 2 + day_x_offset(day, day_of_week);

      let mut style = Style::default();

      if current_date == today {
        style = style.add_modifier(Modifier::UNDERLINED);
      };
      if day == state.datetime.day() {
        style = style.bg(Color::Red);
      };

      buf.set_string(x, y, format!("{day}"), style);

      if day == days_in_month {
        printing = false;
      } else {
        if day_of_week == Weekday::Sat {
          printing_week += 1;
        }
        current_date += Duration::try_days(1).unwrap();
      }
    }
  }
}

impl<'a> Widget for Calendar<'a> {
  fn render(self, area: Rect, buf: &mut Buffer) {
    let mut state = CalendarState::today();
    StatefulWidget::render(self, area, buf, &mut state);
  }
}

fn centered_line_offset(text_width: u16, area_width: u16, area_offset: u16) -> u16 {
  (area_width / 2).saturating_sub(text_width / 2) + area_offset
}

fn overheader(buf: &mut Buffer, area: Rect, datetime: NaiveDate) {
  let text = datetime.format("%-d %B %C%y").to_string();
  let x = centered_line_offset(text.len().try_into().unwrap(), area.width, area.x);
  buf.set_string(x, area.y, text, Style::default());
}

fn header(buf: &mut Buffer, area: Rect, _datetime: NaiveDate) {
  let header_style = Style::default().add_modifier(Modifier::UNDERLINED);
  let text = "Su Mo Tu We Th Fr Sa";
  let x = centered_line_offset(text.len().try_into().unwrap(), area.width, area.x);
  buf.set_string(x, area.y + 1, text, header_style);
}

fn day_x_offset(day: u32, weekday: Weekday) -> u16 {
  let weekday_offset = match weekday {
    Weekday::Sun => 0,
    Weekday::Mon => 3,
    Weekday::Tue => 6,
    Weekday::Wed => 9,
    Weekday::Thu => 12,
    Weekday::Fri => 15,
    Weekday::Sat => 18,
  };

  if day < 10 {
    weekday_offset + 1
  } else {
    weekday_offset
  }
}

fn total_days_in_month(year: i32, month: u32) -> Option<u32> {
  let days = NaiveDate::from_ymd_opt(
    match month {
      12 => year + 1,
      _ => year,
    },
    match month {
      12 => 1,
      _ => month + 1,
    },
    1,
  )
  .unwrap()
  .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1).unwrap())
  .num_days();

  u32::try_from(days).ok()
}
